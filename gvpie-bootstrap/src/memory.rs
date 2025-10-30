use std::collections::{BTreeMap, HashMap};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AddressSpaceId(u32);

#[derive(Default)]
struct Segment {
    base: u64,
    data: Vec<u8>,
}

#[derive(Default)]
struct AddressSpace {
    segments: BTreeMap<u64, Segment>,
}

pub struct Layer4Memory {
    next_id: u32,
    spaces: HashMap<AddressSpaceId, AddressSpace>,
    lineage: Vec<String>,
}

impl Layer4Memory {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            spaces: HashMap::new(),
            lineage: Vec::new(),
        }
    }

    pub fn create_address_space(&mut self) -> AddressSpaceId {
        let id = AddressSpaceId(self.next_id);
        self.next_id += 1;
        self.spaces.insert(id, AddressSpace::default());
        self.lineage.push(format!("create address space {:?}", id));
        id
    }

    pub fn map_and_write(
        &mut self,
        id: AddressSpaceId,
        guest_addr: u64,
        bytes: &[u8],
    ) -> Result<(), String> {
        let space = self
            .spaces
            .get_mut(&id)
            .ok_or_else(|| format!("address space {:?} missing", id))?;
        let segment = space
            .segments
            .entry(guest_addr)
            .or_insert_with(|| Segment {
                base: guest_addr,
                data: vec![0u8; bytes.len()],
            });
        if segment.data.len() < bytes.len() {
            segment.data.resize(bytes.len(), 0);
        }
        segment.data[..bytes.len()].copy_from_slice(bytes);
        self.lineage.push(format!(
            "map+write {:?}: addr=0x{guest_addr:08x}, len={}",
            id,
            bytes.len()
        ));
        Ok(())
    }

    pub fn read(&self, id: AddressSpaceId, guest_addr: u64, len: usize) -> Vec<u8> {
        if let Some(space) = self.spaces.get(&id) {
            if let Some((_, seg)) = space.segments.range(..=guest_addr).next_back() {
                if guest_addr >= seg.base {
                    let offset = (guest_addr - seg.base) as usize;
                    if offset + len <= seg.data.len() {
                        return seg.data[offset..offset + len].to_vec();
                    }
                }
            }
        }
        vec![0u8; len]
    }

    pub fn write(&mut self, id: AddressSpaceId, guest_addr: u64, data: &[u8]) {
        if let Some(space) = self.spaces.get_mut(&id) {
            if let Some((base, seg)) = space.segments.range_mut(..=guest_addr).next_back() {
                if guest_addr >= *base {
                    let offset = (guest_addr - seg.base) as usize;
                    let end = offset + data.len();
                    if end <= seg.data.len() {
                        seg.data[offset..end].copy_from_slice(data);
                        return;
                    }
                }
            }
        }
    }

    pub fn lineage(&mut self) -> &mut Vec<String> {
        &mut self.lineage
    }
}
