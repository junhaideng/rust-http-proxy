use shell::{cmd, ShellError};

// 调用Iptable 使得流量转发带代理服务器上
// iptables -t nat -I OUTPUT -p tcp --dport 80 -j REDIRECT --to-ports 8080
// iptables -t nat -I OUTPUT -p tcp --dport 80 -m owner --uid-owner root -j ACCEPT
// 代理服务器在 root 权限下便可代理其他非 root 用户的流量
pub fn init(port: &str) -> Result<(), ShellError> {
    cmd!(format!(
        "iptables -t nat -I OUTPUT -p tcp --dport 80 -j REDIRECT --to-ports {}",
        &port
    )
    .as_str())
    .run()?;
    cmd!("iptables -t nat -A OUTPUT -p tcp --dport 80 -m owner --uid-owner root -j ACCEPT")
        .run()?;
    Ok(())
}
