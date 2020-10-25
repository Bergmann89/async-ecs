use std::collections::hash_map::{Entry, HashMap};
use std::fmt::Debug;

use tokio::{
    sync::watch::channel,
    task::{spawn as spawn_task, spawn_local},
};

use crate::{access::Accessor, resource::ResourceId, system::AsyncSystem};

use super::{
    task::{execute_local, execute_thread},
    Dispatcher, Error, LocalRun, Receiver, Sender, SharedWorld, ThreadRun,
};

pub struct Builder {
    next_id: SystemId,
    items: HashMap<SystemId, Item>,
    names: HashMap<String, SystemId>,
}

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

enum RunType {
    Thread(ThreadRun),
    Local(LocalRun),
}

#[derive(Default, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct SystemId(pub usize);

impl Builder {
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
            };
        }

        Dispatcher {
            sender,
            receivers,
            world,
        }
    }

    pub fn with<S>(mut self, system: S, name: &str, dependencies: &[&str]) -> Result<Self, Error>
    where
        S: for<'s> AsyncSystem<'s> + Send + 'static,
    {
        self.add(system, name, dependencies)?;

        Ok(self)
    }

    pub fn add<S>(
        &mut self,
        system: S,
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
            |this, id| match this.items.entry(id) {
                Entry::Vacant(e) => e.insert(Item::thread(name.into(), system)),
                Entry::Occupied(_) => panic!("Item was already created!"),
            },
        )?;

        Ok(self)
    }

    pub fn with_local<S>(
        mut self,
        system: S,
        name: &str,
        dependencies: &[&str],
    ) -> Result<Self, Error>
    where
        S: for<'s> AsyncSystem<'s> + 'static,
    {
        self.add_local(system, name, dependencies)?;

        Ok(self)
    }

    pub fn add_local<S>(
        &mut self,
        system: S,
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
            |this, id| match this.items.entry(id) {
                Entry::Vacant(e) => e.insert(Item::local(name.into(), system)),
                Entry::Occupied(_) => panic!("Item was already created!"),
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

impl Default for Builder {
    fn default() -> Self {
        Self {
            next_id: SystemId(0),
            items: HashMap::new(),
            names: HashMap::new(),
        }
    }
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
        S: for<'s> AsyncSystem<'s> + Send + 'static,
    {
        Self::new(name, RunType::Thread(Box::new(system)))
    }

    fn local<S>(name: String, system: S) -> Self
    where
        S: for<'s> AsyncSystem<'s> + 'static,
    {
        Self::new(name, RunType::Local(Box::new(system)))
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

        fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self> {
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
