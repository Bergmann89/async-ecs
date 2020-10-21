use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy)]
pub struct Entity(EntityRaw);

pub type Index = u32;
pub type Generation = u32;

#[repr(C, packed)]
#[derive(Clone, Copy)]
union EntityRaw {
    id: u64,
    data: EntityData,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct EntityData {
    index: Index,
    generation: Generation,
}

impl Entity {
    pub fn from_id(id: u64) -> Self {
        Self(EntityRaw { id })
    }

    pub fn from_parts(index: Index, generation: Generation) -> Self {
        Self(EntityRaw {
            data: EntityData { index, generation },
        })
    }

    #[inline]
    pub fn id(&self) -> u64 {
        unsafe { self.0.id }
    }

    #[inline]
    pub fn index(&self) -> Index {
        unsafe { self.0.data.index }
    }

    #[inline]
    pub fn generation(&self) -> Generation {
        unsafe { self.0.data.generation }
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:08X}", self.id())
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:08X}", self.id())
    }
}

impl Hash for Entity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index().hash(state);
        self.generation().hash(state);
    }
}

impl Eq for Entity {}

impl PartialEq<Entity> for Entity {
    fn eq(&self, other: &Entity) -> bool {
        self.id() == other.id()
    }
}

impl Ord for Entity {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.generation() < other.generation() {
            Ordering::Less
        } else if self.generation() > other.generation() {
            Ordering::Greater
        } else if self.index() < other.index() {
            Ordering::Less
        } else if self.index() > other.index() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for Entity {
    fn partial_cmp(&self, other: &Entity) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}
