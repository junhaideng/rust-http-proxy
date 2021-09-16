use iptables;
use std::error::Error;

// 调用Iptable 使得流量转发带代理服务器上
// TODO
pub struct Iptable();

impl Iptable {
    pub fn init(port: u16) -> Result<(), Box<dyn Error>> {
        let ipt = iptables::new(false)?;
        // 进行DNAT
        ipt.append(
            "nat",
            "PREROUTING",
            &format!("–p tcp –j REDIRECT –to –ports {}", port),
        )
        .unwrap();
        Ok(())
    }
}
