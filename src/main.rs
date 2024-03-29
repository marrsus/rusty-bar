use rusty_bar::clock::Clock;
use rusty_bar::battery::{Battery,BatteryInfo,Status};
use rusty_bar::cpu::Cpu;
use rusty_bar::active_window_title::ActiveWindowTitle;
use rusty_bar::leftwm::{LeftWM,LeftWMAttributes};
use rusty_bar::disk_usage::{DiskUsage,DiskInfo};
use rusty_bar::text::{Font,Color,Attributes,Padding,Threshold};
use rusty_bar::wireless::Wireless;
use rusty_bar::sensors::Sensors;
use rusty_bar::volume::Volume;
use rusty_bar::widget::Cnx;
use rusty_bar::bar::Position;
use anyhow::Result;



fn template(icon:String,info:String) -> String{
    let c1="#00ee00";
    let c2="#eeeeee";
    format!("<span foreground=\"{c1}\">{icon}</span><span foreground=\"{c2}\">{info}</span>")
}



fn attr() -> Attributes {
    Attributes{
	font: Font::new("Hack Nerd Font 11"),
	fg_color: Color::from_hex("#eeeeee"),
	bg_color: None,
	padding: Padding::new(8.0, 8.0, 0.0, 0.0),
    }
}



fn main() -> Result<()> {
    let focused = Attributes {
        fg_color: Color::from_hex("#55ff55"),
	padding: Padding::new(8.0, 8.0, 0.0, 0.0),
	bg_color: Some(Color::from_hex("#222222")),
	 ..attr()
        };
    let visible = Attributes {
        fg_color: Color::green(),
        ..attr()
    };
    let busy = Attributes {
	fg_color: Color::from_hex("#119911"),
	padding: Padding::new(1.0, 1.0, 0.0, 0.0),
	..attr()
    };
    let empty = Attributes {
	fg_color: Color::from_hex("#bbbbbb"),
	padding: Padding::new(1.0, 1.0, 0.0, 0.0),
	..attr()
    };
    
    let pager = LeftWM::new("eDP-1".to_string(),LeftWMAttributes {focused,visible,busy,empty});


    let battery_render = Box::new(|battery_info: BatteryInfo| {
        let percentage = battery_info.capacity;
	let mut icon="";
	match battery_info.status {
	    Status::Full=> icon="  ",
	    Status::Charging=> icon="  ",
	    Status::Discharging=> icon="  ",
	    Status::Unknown=> icon="  ",
	}
	
        let default_text = format!("{percentage:.0}%");
	template(String::from(icon),default_text)
   });

    let battery = Battery::new(attr(), Color::red(), Some(String::from("BAT1")), Some(battery_render));

   
    let root_render=Box::new(|disk_info: DiskInfo| {
        let left = (disk_info.used.get_bytes()*100)/(disk_info.total.get_bytes());
        let disk_text = format!("{left}%");
        template(String::from(" "), disk_text)
    });
    let root = DiskUsage::new(attr(), String::from("/"), Some(root_render)); 


    let home_render=Box::new(|disk_info: DiskInfo| {
        let left = (disk_info.used.get_bytes()*100)/(disk_info.total.get_bytes());
        let disk_text = format!("{left}%");
        template(String::from(" "), disk_text)
    });
    let home = DiskUsage::new(attr(), String::from("/home"), Some(home_render)); 


    let cpu_render = Box::new(|load| {
	
        template(String::from(" "),format!("{load:.0}%"))
    });
    let cpu = Cpu::new(attr(), Some(cpu_render))?;
    

    let mut cnx = Cnx::new(Position::Top);
    
    cnx.add_widget(pager);
    cnx.add_widget(ActiveWindowTitle::new(attr()));
    cnx.add_widget(cpu);
    cnx.add_widget(root);
    cnx.add_widget(home);
    cnx.add_widget(Wireless::new(attr(),String::from("wlan0"),Some(Threshold::default())));
    cnx.add_widget(Sensors::new(attr(),vec!["Package id 0"]));
    cnx.add_widget(Volume::new(attr()));
    cnx.add_widget(battery);

    cnx.add_widget(Clock::new(attr(),Some(String::from("%d-%m-%Y %a %H:%M"))));
    cnx.run()?;

    Ok(())
    }
