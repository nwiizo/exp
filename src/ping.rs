use std::net::{IpAddr, ToSocketAddrs};
use tokio::process::Command;

fn is_ipv6_addr(target: &str) -> bool {
    if let Ok(addr) = target.parse::<IpAddr>() {
        return addr.is_ipv6();
    }

    if let Ok(mut addrs) = format!("{}:0", target).to_socket_addrs() {
        if let Some(addr) = addrs.next() {
            return addr.is_ipv6();
        }
    }
    false
}

fn extract_latency(output: &str) -> Option<f64> {
    // "round-trip min/avg/max/stddev = 9.666/9.666/9.666/nan ms" からの抽出
    if let Some(stats_line) = output.lines().find(|line| line.contains("round-trip")) {
        if let Some(values) = stats_line.split('=').nth(1) {
            if let Some(avg) = values.split('/').nth(1) {
                return avg.trim().parse::<f64>().ok();
            }
        }
    }
    None
}

pub async fn ping(target: &str) -> (bool, Option<f64>) {
    let is_ipv6 = is_ipv6_addr(target);

    let output = if is_ipv6 {
        Command::new("ping6")
            .arg("-c")
            .arg("1")
            .arg(target)
            .output()
            .await
    } else {
        Command::new("ping")
            .arg("-c")
            .arg("1")
            .arg(target)
            .output()
            .await
    };

    match output {
        Ok(output) => {
            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout);

            // pingが成功した場合のみレイテンシーを抽出
            let latency = if success {
                extract_latency(&stdout)
            } else {
                None
            };

            (success, latency)
        }
        Err(_) => (false, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() {
        let ipv4_result = ping("1.1.1.1").await;
        println!("IPv4 (1.1.1.1) result: {:?}", ipv4_result);

        let ipv6_result = ping("2606:4700:4700::1111").await;
        println!("IPv6 (2606:4700:4700::1111) result: {:?}", ipv6_result);
    }

    #[test]
    fn test_extract_latency() {
        let sample_output = "round-trip min/avg/max/stddev = 9.666/9.666/9.666/nan ms";
        assert_eq!(extract_latency(sample_output), Some(9.666));
    }
}
