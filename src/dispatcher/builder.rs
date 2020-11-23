use std::collections::hash_map::{Entry, HashMap};
use std::fmt::Debug;

use tokio::{
    sync::watch::channel,
    task::{spawn as spawn_task, spawn_local},
};

use crate::{
    access::Accessor,
    resource::ResourceId,
    system::{AsyncSystem, System},
    world::World,
};

use super::{
    task::{execute_local, execute_local_async, execute_thread, execute_thread_async},
    Dispatcher, Error, LocalRun, LocalRunAsync, Receiver, Sender, SharedWorld, ThreadRun,
    ThreadRunAsync,
};

/// Id of a system inside the `Dispatcher` and the `Builder`.
#[derive(Default, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct SystemId(pub usize);

/// Builder for the [`Dispatcher`].
///
/// [`Dispatcher`]: struct.Dispatcher.html
///
/// ## Barriers
///
/// Barriers are a way of sequentializing parts of
/// the system execution. See `add_barrier()`/`with_barrier()`.
///
/// ## Examples
///
/// This is how you create a dispatcher with
/// a shared thread pool:
///
/// ```rust
/// # #![allow(unused)]
/// #
/// # use async_ecs::*;
/// #
/// # #[derive(Debug, Default)]
/// # struct Res;
/// #
/// # #[derive(SystemData)]
/// # struct Data<'a> { a: Read<'a, Res> }
/// #
/// # struct Dummy;
/// #
/// # impl<'a> System<'a> for Dummy {
/// #   type SystemData = Data<'a>;
/// #
/// #   fn run(&mut self, _: Data<'a>) {}
/// # }
/// #
/// # #[tokio::main]
/// # async fn main() {
/// # let system_a = Dummy;
/// # let system_b = Dummy;
/// # let system_c = Dummy;
/// # let system_d = Dummy;
/// # let system_e = Dummy;
/// #
/// let dispatcher = Dispatcher::builder()
///     .with(system_a, "a", &[])
///     .unwrap()
///     .with(system_b, "b", &["a"])
///     .unwrap() // b depends on a
///     .with(system_c, "c", &["a"])
///     .unwrap() // c also depends on a
///     .with(system_d, "d", &[])
///     .unwrap()
///     .with(system_e, "e", &["c", "d"])
///     .unwrap() // e executes after c and d are finished
///     .build();
/// # }
/// ```
///
/// Systems can be conditionally added by using the `add_` functions:
///
/// ```rust
/// # #![allow(unused)]
/// #
/// # use async_ecs::*;
/// #
/// # #[derive(Debug, Default)]
/// # struct Res;
/// #
/// # #[derive(SystemData)]
/// # struct Data<'a> { a: Read<'a, Res> }
/// #
/// # struct Dummy;
/// #
/// # impl<'a> System<'a> for Dummy {
/// #   type SystemData = Data<'a>;
/// #
/// #   fn run(&mut self, _: Data<'a>) {}
/// # }
/// #
/// # #[tokio::main]
/// # async fn main() {
/// # let b_enabled = true;
/// # let system_a = Dummy;
/// # let system_b = Dummy;
/// let mut builder = Dispatcher::builder().with(system_a, "a", &[]).unwrap();
///
/// if b_enabled {
///     builder.add(system_b, "b", &[]).unwrap();
/// }
///
/// let dispatcher = builder.build();
/// # }
/// ```
pub struct Builder<'a> {
    world: Option<&'a mut World>,
    next_id: SystemId,
    items: HashMap<SystemId, Item>,
    names: HashMap<String, SystemId>,
}

impl<'a> Builder<'a> {
    pub fn new(world: Option<&'a mut World>) -> Self {
        Self {
            world,
            next_id: Default::default(),
            items: Default::default(),
            names: Default::default(),
        }
    }

    /// Builds the `Dispatcher`.
    ///
    /// This method will precompute useful information in order to speed up dispatching.
    pub fn build(self) -> Dispatcher {
        let receivers = self
            .final_systems()
            .into_iter()
            .map(|id| self.items.get(&id).unwrap().receiver.clone())
            .collect();

        let world = SharedWorld::default();
        let (sender, receiver) = channel(());

        for (_, item) in self.items.into_iter() {
            let run = item.run;
            let name = item.name;
            let sender = item.sender;
            let receivers = if item.dependencies.is_empty() {
                vec![receiver.clone()]
            } else {
                item.receivers
            };

            match run {
                RunType::Thread(run) => {
                    spawn_task(execute_thread(name, run, sender, receivers, world.clone()))
                }
                RunType::Local(run) => {
                    spawn_local(execute_local(name, run, sender, receivers, world.clone()))
                }
                RunType::ThreadAsync(run) => spawn_task(execute_thread_async(
                    name,
                    run,
                    sender,
                    receivers,
                    world.clone(),
                )),
                RunType::LocalAsync(run) => spawn_local(execute_local_async(
                    name,
                    run,
                    sender,
                    receivers,
                    world.clone(),
                )),
            };
        }

        Dispatcher {
            sender,
            receivers,
            world,
        }
    }

    /// Adds a new system with a given name and a list of dependencies.
    /// Please note that the dependency should be added before
    /// you add the depending system.
    ///
    /// If you want to register systems which can not be specified as
    /// dependencies, you can use `""` as their name, which will not panic
    /// (using another name twice will).
    ///
    /// Same as [`add()`](struct.Dispatcher::builder().html#method.add), but
    /// returns `self` to enable method chaining.
    pub fn with<S>(mut self, system: S, name: &str, dependencies: &[&str]) -> Result<Self, Error>
    where
        S: for<'s> System<'s> + Send + 'static,
    {
        self.add(system, name, dependencies)?;

        Ok(self)
    }

    /// Adds a new system with a given name and a list of dependencies.
    /// Please note that the dependency should be added before
    /// you add the depending system.
    ///
    /// If you want to register systems which can not be specified as
    /// dependencies, you can use `""` as their name, which will not panic
    /// (using another name twice will).
    pub fn add<S>(
        &mut self,
        mut system: S,
        name: &str,
        dependencies: &[&str],
    ) -> Result<&mut Self, Error>
    where
        S: for<'s> System<'s> + Send + 'static,
    {
        self.add_inner(
            name,
            dependencies,
            system.accessor().reads(),
            system.accessor().writes(),
            |this, id| {
                if let Some(ref mut w) = this.world {
                    system.setup(w)
                }

                match this.items.entry(id) {
                    Entry::Vacant(e) => e.insert(Item::thread(name.into(), system)),
                    Entry::Occupied(_) => panic!("Item was already created!"),
                }
            },
        )?;

        Ok(self)
    }

    /// Adds a new asynchronous system with a given name and a list of dependencies.
    /// Please note that the dependency should be added before
    /// you add the depending system.
    ///
    /// If you want to register systems which can not be specified as
    /// dependencies, you can use `""` as their name, which will not panic
    /// (using another name twice will).
    ///
    /// Same as [`add()`](struct.Dispatcher::builder().html#method.add), but
    /// returns `self` to enable method chaining.
    pub fn with_async<S>(
        mut self,
        system: S,
        name: &str,
        dependencies: &[&str],
    ) -> Result<Self, Error>
    where
        S: for<'s> AsyncSystem<'s> + Send + 'static,
    {
        self.add_async(system, name, dependencies)?;

        Ok(self)
    }

    /// Adds a new asynchronous system with a given name and a list of dependencies.
    /// Please note that the dependency should be added before
    /// you add the depending system.
    ///
    /// If you want to register systems which can not be specified as
    /// dependencies, you can use `""` as their name, which will not panic
    /// (using another name twice will).
    pub fn add_async<S>(
        &mut self,
        mut system: S,
        name: &str,
        dependencies: &[&str],
    ) -> Result<&mut Self, Error>
    where
        S: for<'s> AsyncSystem<'s> + Send + 'static,
    {
        self.add_inner(
            name,
            dependencies,
            system.accessor().reads(),
            system.accessor().writes(),
            |this, id| {
                if let Some(ref mut w) = this.world {
                    system.setup(w)
                }

                match this.items.entry(id) {
                    Entry::Vacant(e) => e.insert(Item::thread_async(name.into(), system)),
                    Entry::Occupied(_) => panic!("Item was already created!"),
                }
            },
        )?;

        Ok(self)
    }

    /// Adds a new thread local system.
    ///
    /// Please only use this if your struct is not `Send` and `Sync`.
    ///
    /// Thread-local systems are dispatched in-order.
    ///
    /// Same as [Dispatcher::builder()::add_local], but returns `self` to
    /// enable method chaining.
    pub fn with_local<S>(
        mut self,
        system: S,
        name: &str,
        dependencies: &[&str],
    ) -> Result<Self, Error>
    where
        S: for<'s> System<'s> + 'static,
    {
        self.add_local(system, name, dependencies)?;

        Ok(self)
    }

    /// Adds a new thread local system.
    ///
    /// Please only use this if your struct is not `Send` and `Sync`.
    ///
    /// Thread-local systems are dispatched in-order.
    pub fn add_local<S>(
        &mut self,
        mut system: S,
        name: &str,
        dependencies: &[&str],
    ) -> Result<&mut Self, Error>
    where
        S: for<'s> System<'s> + 'static,
    {
        self.add_inner(
            name,
            dependencies,
            system.accessor().reads(),
            system.accessor().writes(),
            |this, id| {
                if let Some(ref mut w) = this.world {
                    system.setup(w)
                }

                match this.items.entry(id) {
                    Entry::Vacant(e) => e.insert(Item::local(name.into(), system)),
                    Entry::Occupied(_) => panic!("Item was already created!"),
                }
            },
        )?;

        Ok(self)
    }

    /// Adds a new thread local asynchronous system.
    ///
    /// Please only use this if your struct is not `Send` and `Sync`.
    ///
    /// Thread-local systems are dispatched in-order.
    ///
    /// Same as [Dispatcher::builder()::add_local], but returns `self` to
    /// enable method chaining.
    pub fn with_local_async<S>(
        mut self,
        system: S,
        name: &str,
        dependencies: &[&str],
    ) -> Result<Self, Error>
    where
        S: for<'s> AsyncSystem<'s> + 'static,
    {
        self.add_local_async(system, name, dependencies)?;

        Ok(self)
    }

    /// Adds a new thread local asynchronous system.
    ///
    /// Please only use this if your struct is not `Send` and `Sync`.
    ///
    /// Thread-local systems are dispatched in-order.
    pub fn add_local_async<S>(
        &mut self,
        mut system: S,
        name: &str,
        dependencies: &[&str],
    ) -> Result<&mut Self, Error>
    where
        S: for<'s> AsyncSystem<'s> + 'static,
    {
        self.add_inner(
            name,
            dependencies,
            system.accessor().reads(),
            system.accessor().writes(),
            |this, id| {
                if let Some(ref mut w) = this.world {
                    system.setup(w)
                }

                match this.items.entry(id) {
                    Entry::Vacant(e) => e.insert(Item::local_async(name.into(), system)),
                    Entry::Occupied(_) => panic!("Item was already created!"),
                }
            },
        )?;

        Ok(self)
    }

    fn add_inner<F>(
        &mut self,
        name: &str,
        dependencies: &[&str],
        mut reads: Vec<ResourceId>,
        mut writes: Vec<ResourceId>,
        f: F,
    ) -> Result<&mut Self, Error>
    where
        F: FnOnce(&mut Self, SystemId) -> &mut Item,
    {
        let name = name.to_owned();
        let id = self.next_id();
        let id = match self.names.entry(name) {
            Entry::Vacant(e) => Ok(*e.insert(id)),
            Entry::Occupied(e) => Err(Error::NameAlreadyRegistered(e.key().into())),
        }?;

        reads.sort();
        writes.sort();

        reads.dedup();
        writes.dedup();

        let mut dependencies = dependencies
            .iter()
            .map(|name| {
                self.names
                    .get(*name)
                    .map(Clone::clone)
                    .ok_or_else(|| Error::DependencyWasNotFound((*name).into()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        for read in &reads {
            for (key, value) in &self.items {
                if value.writes.contains(read) {
                    dependencies.push(*key);
                }
            }
        }

        for write in &writes {
            for (key, value) in &self.items {
                if value.reads.contains(write) || value.writes.contains(write) {
                    dependencies.push(*key);
                }
            }
        }

        self.reduce_dependencies(&mut dependencies);

        let receivers = dependencies
            .iter()
            .map(|id| self.items.get(id).unwrap().receiver.clone())
            .collect();

        let item = f(self, id);

        item.reads = reads;
        item.writes = writes;
        item.receivers = receivers;
        item.dependencies = dependencies;

        Ok(self)
    }

    fn final_systems(&self) -> Vec<SystemId> {
        let mut ret = self.items.keys().map(Clone::clone).collect();

        self.reduce_dependencies(&mut ret);

        ret
    }

    fn reduce_dependencies(&self, dependencies: &mut Vec<SystemId>) {
        dependencies.sort();
        dependencies.dedup();

        let mut remove_indices = Vec::new();
        for (i, a) in dependencies.iter().enumerate() {
            for (j, b) in dependencies.iter().enumerate() {
                if self.depends_on(a, b) {
                    remove_indices.push(j);
                } else if self.depends_on(b, a) {
                    remove_indices.push(i);
                }
            }
        }

        remove_indices.sort_unstable();
        remove_indices.dedup();
        remove_indices.reverse();

        for i in remove_indices {
            dependencies.remove(i);
        }
    }

    fn depends_on(&self, a: &SystemId, b: &SystemId) -> bool {
        let item = self.items.get(a).unwrap();

        if item.dependencies.contains(b) {
            return true;
        }

        for d in &item.dependencies {
            if self.depends_on(d, b) {
                return true;
            }
        }

        false
    }

    fn next_id(&mut self) -> SystemId {
        self.next_id.0 += 1;

        self.next_id
    }
}

/// Defines how to execute the `System` with the `Dispatcher`.
enum RunType {
    Thread(ThreadRun),
    Local(LocalRun),
    ThreadAsync(ThreadRunAsync),
    LocalAsync(LocalRunAsync),
}

/// Item that wraps all information of a 'System` within the `Builder`.
struct Item {
    name: String,
    run: RunType,

    sender: Sender,
    receiver: Receiver,
    receivers: Vec<Receiver>,

    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    dependencies: Vec<SystemId>,
}

impl Item {
    fn new(name: String, run: RunType) -> Self {
        let (sender, receiver) = channel(());

        Self {
            name,
            run,

            sender,
            receiver,
            receivers: Vec::new(),

            reads: Vec::new(),
            writes: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    fn thread<S>(name: String, system: S) -> Self
    where
        S: for<'s> System<'s> + Send + 'static,
    {
        Self::new(name, RunType::Thread(Box::new(system)))
    }

    fn local<S>(name: String, system: S) -> Self
    where
        S: for<'s> System<'s> + 'static,
    {
        Self::new(name, RunType::Local(Box::new(system)))
    }

    fn thread_async<S>(name: String, system: S) -> Self
    where
        S: for<'s> AsyncSystem<'s> + Send + 'static,
    {
        Self::new(name, RunType::ThreadAsync(Box::new(system)))
    }

    fn local_async<S>(name: String, system: S) -> Self
    where
        S: for<'s> AsyncSystem<'s> + 'static,
    {
        Self::new(name, RunType::LocalAsync(Box::new(system)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        access::AccessorCow,
        system::{DynamicSystemData, System},
        world::World,
    };

    #[test]
    fn dependencies_on_read_and_write() {
        /*
            - Systems ------------------------------------
                Id:     1       2       3       4       5

            - Resources ----------------------------------
                Read:   A       A       B       C       A
                Write:  B       C       D       D      BCD

            - Dependencies Total -------------------------
                        |       |       |       |       |
                        |<--------------|       |       |
                        |       |       |       |       |
                        |       |<--------------|       |
                        |       |       |<------|       |
                        |       |       |       |       |
                        |<------------------------------|
                        |       |<----------------------|
                        |       |       |<--------------|
                        |       |       |       |<------|
                        |       |       |       |       |

            - Dependencies Reduced -----------------------
                        |       |       |       |       |
                        |<--------------|       |       |
                        |       |       |       |       |
                        |       |<--------------|       |
                        |       |       |<------|       |
                        |       |       |       |       |
                        |       |       |       |<------|
                        |       |       |       |       |
        */

        struct ResA;
        struct ResB;
        struct ResC;
        struct ResD;

        let sys1 = TestSystem::new(
            vec![ResourceId::new::<ResA>()],
            vec![ResourceId::new::<ResB>()],
        );
        let sys2 = TestSystem::new(
            vec![ResourceId::new::<ResA>()],
            vec![ResourceId::new::<ResC>()],
        );
        let sys3 = TestSystem::new(
            vec![ResourceId::new::<ResB>()],
            vec![ResourceId::new::<ResD>()],
        );
        let sys4 = TestSystem::new(
            vec![ResourceId::new::<ResC>()],
            vec![ResourceId::new::<ResD>()],
        );
        let sys5 = TestSystem::new(
            vec![ResourceId::new::<ResA>()],
            vec![
                ResourceId::new::<ResB>(),
                ResourceId::new::<ResC>(),
                ResourceId::new::<ResD>(),
            ],
        );

        let dispatcher = Dispatcher::builder()
            .with(sys1, "sys1", &[])
            .unwrap()
            .with(sys2, "sys2", &[])
            .unwrap()
            .with(sys3, "sys3", &[])
            .unwrap()
            .with(sys4, "sys4", &[])
            .unwrap()
            .with(sys5, "sys5", &[])
            .unwrap();

        let sys1 = dispatcher.items.get(&SystemId(1)).unwrap();
        let sys2 = dispatcher.items.get(&SystemId(2)).unwrap();
        let sys3 = dispatcher.items.get(&SystemId(3)).unwrap();
        let sys4 = dispatcher.items.get(&SystemId(4)).unwrap();
        let sys5 = dispatcher.items.get(&SystemId(5)).unwrap();

        assert_eq!(sys1.dependencies, vec![]);
        assert_eq!(sys2.dependencies, vec![]);
        assert_eq!(sys3.dependencies, vec![SystemId(1)]);
        assert_eq!(sys4.dependencies, vec![SystemId(2), SystemId(3)]);
        assert_eq!(sys5.dependencies, vec![SystemId(4)]);
        assert_eq!(dispatcher.final_systems(), vec![SystemId(5)]);
    }

    struct TestSystem {
        accessor: TestAccessor,
    }

    impl TestSystem {
        fn new(reads: Vec<ResourceId>, writes: Vec<ResourceId>) -> Self {
            Self {
                accessor: TestAccessor { reads, writes },
            }
        }
    }

    impl<'a> System<'a> for TestSystem {
        type SystemData = TestData;

        fn run(&mut self, _data: Self::SystemData) {
            unimplemented!()
        }

        fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self::SystemData> {
            AccessorCow::Borrow(&self.accessor)
        }
    }

    struct TestData;

    impl<'a> DynamicSystemData<'a> for TestData {
        type Accessor = TestAccessor;

        fn setup(_accessor: &Self::Accessor, _world: &mut World) {}

        fn fetch(_access: &Self::Accessor, _world: &'a World) -> Self {
            TestData
        }
    }

    struct TestAccessor {
        reads: Vec<ResourceId>,
        writes: Vec<ResourceId>,
    }

    impl Accessor for TestAccessor {
        fn reads(&self) -> Vec<ResourceId> {
            self.reads.clone()
        }

        fn writes(&self) -> Vec<ResourceId> {
            self.writes.clone()
        }
    }
}
