extern crate capnp;
extern crate core;
extern crate capnp2arrow;

//use capnp::serialize_packed;
use capnp::dynamic_value;
use capnp2arrow::map_schema;

pub mod point_capnp {
    include!(concat!(env!("OUT_DIR"), "/point_capnp.rs"));
}

fn main() {
    let mut message = ::capnp::message::Builder::new_default();

    let mut demo_point = message.init_root::<point_capnp::point::Builder>();

    demo_point.set_x(5_f32);
    demo_point.set_y(10_f32);

    let reader = demo_point.into_reader();

    println!("{:?}", reader);

    //serialize_packed::write_message(&mut ::std::io::stdout(), &message);

    let dynamic: dynamic_value::Reader = reader.into();
    let schema = map_schema(dynamic.downcast());

    println!("{:?}", schema);
}
