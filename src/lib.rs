use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::net::{SocketAddr, TcpStream};
use log::*;
use serde::Deserialize;
use serde_json::*;
use serde_json::value::Value;

extern crate log;
extern crate serde;
extern crate serde_json;

pub enum Prop {
    ActiveMode(),
    BgBrightness(Brightness),
    BgColorMode(ColorMode),
    // BgCt(ColorTemp), TODO
    BgFlowParams(FlowAction, Vec<FlowExpression>),
    BgFlowing(bool),
    BgPower(bool),
    Brightness(Brightness),
    Color(Color),
    ColorMode(ColorMode),
    // Ct(ColorTemp), TODO
    DelayOff(u32),
    FlowParams(FlowAction, Vec<FlowExpression>),
    Flowing(bool),
    MusicOn(bool),
    Name(String),
    NlBr(Brightness),
    Power(bool),
}

pub struct Percentage(u32);

impl Percentage {
    pub fn create(percentage: u32) -> Option<Percentage> {
        match percentage {
            0..=100 => Some(Percentage(percentage)),
            _ => None,
        }
    }
}

pub struct Delay(u32);

impl Delay {
    pub fn create(delay: u32) -> Option<Delay> {
        Some(Delay(delay))
    }
}

pub struct TransitionDuration(u32);

impl TransitionDuration {
    pub fn create(transition_duration: u32) -> Option<TransitionDuration> {
        Some(TransitionDuration(transition_duration))
    }
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

pub struct Brightness(u32);

impl Brightness {
    pub fn create(brightness: u32) -> Option<Brightness> {
        match brightness {
            0..=100 => Some(Brightness(brightness)),
            _ => None,
        }
    }
}

pub enum Mode {
    Daylight = 0,
    Moonlight = 1,
}

pub enum ColorMode {
    Rgb = 1,
    Ct = 2,
    Hsv = 3,
}

impl Color {
    pub fn create_rgb(red: u8, green: u8, blue: u8) -> Option<Color> {
        Some(Color::Rgb(red, green, blue))
    }

    pub fn create_temp(temp: u32) -> Option<Color> {
        match temp {
            1700..=6500 => Some(Color::Temp(temp)),
            _ => None,
        }
    }

    pub fn create_hsv(hue: u32, saturation: u32) -> Option<Color> {
        match (hue, saturation) {
            (0..=100, 0..=359) => Some(Color::Hsv(hue, saturation)),
            _ => None,
        }
    }
}

pub enum Color {
    Rgb(u8, u8, u8),
    Temp(u32),
    Hsv(u32, u32),
}

pub struct Yeelight {
    sock: SocketAddr,
    stream: TcpStream,
    next_id: u32,
}

pub enum Effect {
    Sudden,
    Smooth,
}

pub enum FlowAction {
    Recover = 0,
    Stay = 1,
    TurnOff = 2,
}

pub enum FlowMode {
    Color = 1,
    ColorTemperature = 2,
    Sleep = 7,
}

pub struct FlowExpression {
    duration: TransitionDuration,
    value: Option<Color>,
    brightness: Option<Brightness>,
}

pub enum CronType {
    TurnOff = 0,
}

pub enum AdjustAction {
    Increase,
    Decrease,
    Circle,
}

pub enum AdjustProp {
    Bright,
    Ct,
    Color,
}

pub enum SceneClass {
    Color,
    Hsv,
    Ct,
    Flow,
    AutoDelayOff,
}

#[derive(Deserialize)]
struct Response {
    id: u32,
    result: Vec<String>,
}

impl Yeelight {
    pub fn connect(sock: &SocketAddr) -> Result<Yeelight> {
        let stream = TcpStream::connect(sock)?;
        let light = Yeelight {
            sock: sock.clone(),
            stream: stream,
            next_id: 0,
        };
        Ok(light)
    }

    pub fn get_prop(&self) -> Result<&[u8]> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_color(
        &mut self,
        color: Color,
        effect: Effect,
        duration: TransitionDuration,
    ) -> Result<()> {
        let id = self.next_id;
        self.next_id += 1;

        let method = match color {
            Color::Rgb(_, _, _) => "set_rgb",
            Color::Temp(_) => "set_ct_abx",
            Color::Hsv(_, _) => "set_hsv",
        };

        let mut params = Value::from(match color {
            Color::Rgb(red, green, blue) => vec![(red as u32 * 65536 + green as u32 * 256 + blue as u32)],
            Color::Temp(temp) => vec![temp; 3],
            Color::Hsv(hue, saturation) => vec![hue, saturation],
        });

        params.as_array_mut().unwrap().push(Value::from(match effect {
            Effect::Sudden => "sudden",
            Effect::Smooth => "smooth",
        }));

        params.as_array_mut().unwrap().push(Value::from(duration.as_u32()));

        let message = json!({
            "id": id,
            "method": method,
            "params": params,
        });

        debug!("Sending \"{}\" to {}...", message.to_string(), self.sock);

        serde_json::to_writer(&mut self.stream, &message)?;
        self.stream.write(b"\r\n");

        debug!("Sent to {}!", self.sock);

        let mut done = false;
        let mut reader = BufReader::new(&mut self.stream);
        while !done {
            let mut buf = Vec::new();

            debug!("Receiving from {}...", self.sock);
            let ret = reader.read_until('\n' as u8, &mut buf)?;

            // Remove trailing \r\n
            buf.pop();
            buf.pop();

            debug!("Received \"{}\" from {}!", String::from_utf8(buf.clone()).unwrap(), self.sock);

            if ret == 0 {
                done = true;
                return Err(Error::new(ErrorKind::Other, "Received no response"));
            }

            done = match serde_json::from_slice::<Response>(&buf) {
                Ok(res) => res.id == id,
                Err(err) => if err.is_data() { true } else { return Err(err.into()) }
            };
        }

        Ok(())
    }
    pub fn bg_set_color(
        &self,
        color: Color,
        effect: Effect,
        duration: TransitionDuration,
    ) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_bright(
        &self,
        brightness: Brightness,
        effect: Effect,
        duration: TransitionDuration,
    ) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_set_bright(
        &self,
        brightness: Brightness,
        effect: Effect,
        duration: TransitionDuration,
    ) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn toggle(&self) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_toggle(&self) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn dev_toggle(&self) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_power(
        &self,
        power: String,
        effect: Effect,
        duration: TransitionDuration,
    ) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn set_default(&self) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn start_flow(&self, action: FlowAction, flow_params: Vec<FlowExpression>) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn stop_flow(&self) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn set_scene(&self, class: SceneClass) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    } // TODO
    pub fn cron_add(&self, action: CronType, value: Delay) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    } // TODO: Find a better name for action
    pub fn cron_get(&self, action: CronType) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn cron_del(&self, action: CronType) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn set_adjust(&self, action: AdjustAction, prop: AdjustProp) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_music(&self, host: SocketAddr) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn stop_music(&self) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_name(&self, name: String) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn adjust_bright(
        &self,
        percentage: Percentage,
        duration: TransitionDuration,
    ) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn adjust_ct(&self, percentage: Percentage, duration: TransitionDuration) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn adjust_color(&self, percentage: Percentage, duration: TransitionDuration) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_adjust_ct(&self, percentage: Percentage, duration: TransitionDuration) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_adjust_color(
        &self,
        percentage: Percentage,
        duration: TransitionDuration,
    ) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_set_power(
        &self,
        power: String,
        effect: Effect,
        duration: TransitionDuration,
    ) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_set_default(&self) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_start_flow(
        &self,
        action: FlowAction,
        flow_params: Vec<FlowExpression>,
    ) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_stop_flow(&self) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_set_scene(&self, class: SceneClass) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
    pub fn bg_set_adjust(&self, action: AdjustAction, prop: AdjustProp) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
