
    #[derive(PartialEq, Copy, Clone)]
    pub enum PassthroughState{
    Blocked, // channel 1 won't be let through
    Unblocked,
}

// #[derive(Debug)]
pub enum LexerState {
    UbxLeader1, // first constant leader byte found
    UbxLeader2, // second constant leader byte found
    UbxClassId, // classid read
    UbxMessageId, // message id read
    UbxLength1, // first length byte read (le)
    UbxLength2, // second length byte read (le)
    UbxPayload, // payload eating
    UbxChecksumA, // checksum A byte (tcp checksum)
    UbxRecognized, // this is also UBX_CHECKSUM_B
    NmeaDollar,
    NmeaPubLead,
    NmeaCr,
    NmeaRecognized,
    GroundState,
}
// why that?
#[allow(dead_code)]
fn main() {}
