#[cfg(test)]
use image::{ImageBuffer, Luma};
#[cfg(test)]
use std::{
    net::Ipv4Addr,
    thread,
    time::{Duration, Instant},
};

use cameleon::gige::enumerate_cameras;
use pnet::datalink;
use std::net::IpAddr;

fn main() {
    let interfaces = datalink::interfaces();

    for iface in interfaces {
        for ip_net in iface.ips {
            if let IpAddr::V4(ip) = ip_net.ip() {
                println!("🔍 Testing interface IP: {}", ip);
                match enumerate_cameras(ip) {
                    Ok(mut cameras) if !cameras.is_empty() => {
                        println!("🎥 Found camera on {}", ip);
                        let mut camera = cameras.pop().unwrap();
                        camera.open().unwrap();
                        camera.load_context().unwrap();
                        camera.close().unwrap();
                        return;
                    }
                    Ok(_) => println!("❌ No camera found on {}", ip),
                    Err(e) => println!("⚠️ Error on {}: {}", ip, e),
                }
            }
        }
    }

    println!("🚫 No camera found on any local interface");
}

#[test]
fn test_capture_and_save_image() {
    let local_add = Ipv4Addr::new(10, 227, 12, 10);
    let mut cameras = enumerate_cameras(local_add).unwrap();
    let mut camera = cameras.pop().expect("No camera found");

    camera.open().unwrap();
    camera.load_context().unwrap();

    let info = camera.info();
    println!("📷 Kamera: {} {}", info.vendor_name, info.model_name);
    println!("🔢 Seriennummer: {}", info.serial_number);

    let start = Instant::now();
    let timeout = Duration::from_secs(5);

    let stream = camera.start_streaming(2).unwrap();
    let payload = loop {
        match stream.try_recv() {
            Ok(p) => break p,
            Err(_) => {
                if start.elapsed() > timeout {
                    panic!("⏰ Timeout: Kein Bild innerhalb von 5 Sekunden empfangen.");
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    };

    let image_info = payload.image_info().unwrap();
    let image = payload.image().unwrap();

    let buffer = ImageBuffer::<Luma<u8>, _>::from_raw(
        image_info.width as u32,
        image_info.height as u32,
        image,
    )
    .unwrap();
    buffer.save("test_output.png").unwrap();

    camera.close().unwrap();
}
