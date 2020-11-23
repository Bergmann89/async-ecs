use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};

/// `Entity` type, as seen by the user.
#[derive(Clone, Copy)]
pub struct Entity(EntityRaw);

/// Index of the entity.
pub type Index = u32;

/// Generation of the entity.
pub type Generation = u32;

/// Raw data of the entity.
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
    /// Create new entity with the given ID.
    pub fn from_id(id: u64) -> Self {
        Self(EntityRaw { id })
    }

    /// Create new entity with the given given index and generation.
    pub fn from_parts(index: Index, generation: Generation) -> Self {
        Self(EntityRaw {
            data: EntityData { index, generation },
        })
    }

    /// Get the id of the entity.
    #[inline]
    pub fn id(&self) -> u64 {
        unsafe { self.0.id }
    }

    /// Get the index of the entity.
    #[inline]
    pub fn index(&self) -> Index {
        unsafe { self.0.data.index }
    }

    // Get the generation of the entity.
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
