use std::error::Error;

use iptables;

// 调用Iptable 使得流量转发带代理服务器上
// iptables -t nat -I OUTPUT -p tcp --dport 80 -j REDIRECT --to-ports 8080
// iptables -t nat -I OUTPUT -p tcp --dport 80 -m owner --uid-owner root -j ACCEPT
// 代理服务器在 root 权限下便可代理其他非 root 用户的流量
pub fn init(port: &str) -> Result<(), Box<dyn Error>> {
    let ipt = iptables::new(false)?;
    // 进行DNAT
    ipt.append(
        "nat",
        "OUTPUT",
        &format!("–p tcp --dport 80 -j REDIRECT –-to-port {}", port),
    )?;
    ipt.append(
        "nat",
        "OUTPUT",
        "-p tcp --dport 80 -m owner --uid-owner root -j ACCEPT",
    )?;
    Ok(())
}
