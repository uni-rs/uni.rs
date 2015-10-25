type ConsRingIdx = u32;

#[repr(C)]
pub struct ConsoleInterface {
    input: [u8; 1024],
    output: [u8; 2048],
    in_cons: ConsRingIdx,
    in_prod: ConsRingIdx,
    out_cons: ConsRingIdx,
    out_prod: ConsRingIdx,
}
