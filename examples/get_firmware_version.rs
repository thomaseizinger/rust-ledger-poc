use ledger::TransportNativeHID;
use ledger_apdu::APDUCommand;

fn main() -> anyhow::Result<()> {
    let transport = TransportNativeHID::new()?;

    let result = transport.exchange(&APDUCommand {
        cla: 0xe0,
        ins: 0xc4,
        p1: 0x00,
        p2: 0x00,
        data: Vec::new(),
    })?;

    println!(
        "firmware version: {}.{}.{}",
        result.data[2], result.data[3], result.data[4]
    );

    Ok(())
}
