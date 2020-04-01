mod lib;
use lib::*;
use std::net::*;
use std::env::args;

fn main() {
    //let sock = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 7)), 55443);
    //let light: lib::Yeelight = lib::Yeelight { addr: sock };
    //if let Ok(ret) = light.get_prop() {
    //    println!("ok!");
    //}
    stderrlog::new().module(module_path!()).verbosity(10).init().unwrap();
    let mut argv = args();
    argv.next();
    let red = argv.next().unwrap().parse::<u8>().unwrap();
    let green = argv.next().unwrap().parse::<u8>().unwrap();
    let blue = argv.next().unwrap().parse::<u8>().unwrap();

    let sock = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 7)), 55443);
    let mut light = Yeelight::connect(&sock).unwrap();
    let color = Color::create_rgb(red, green, blue).unwrap();
    let duration = TransitionDuration::create(1000).unwrap();
    light.set_color(color, Effect::Smooth, duration);
}
