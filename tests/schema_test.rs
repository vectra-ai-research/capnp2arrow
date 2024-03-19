use capnp::dynamic_value;
use capnp2arrow::field::Field;
use capnp2arrow::reader::capnp_messages_from_data;
use capnp2arrow::deserialize::infer_fields;
use std::fs;
use std::path::Path;

include! {"../src/test.rs"}

fn get_fields() -> Vec<Field> {
    let data_path = Path::new("tests/test_data/all_types.bin");
    let data = fs::read(data_path).unwrap();

    let readers = capnp_messages_from_data(data);
    let messages: Vec<dynamic_value::Reader> = readers
        .iter()
        .map(|r| {
            r.get_root::<test_all_types_capnp::test_all_types::Reader>()
                .unwrap()
                .into()
        })
        .collect();
    let fields = infer_fields(messages.as_slice()).unwrap();
    fields
}

fn get_union_fields() -> Vec<Field> {
    let data_path = Path::new("tests/test_data/union.bin");
    let data = fs::read(data_path).unwrap();

    let readers = capnp_messages_from_data(data);
    let messages: Vec<dynamic_value::Reader> = readers
        .iter()
        .map(|r| {
            r.get_root::<test_all_types_capnp::test_union::Reader>()
                .unwrap()
                .into()
        })
        .collect();
    let fields = infer_fields(messages.as_slice()).unwrap();
    fields
}

#[test]
fn test_primitives() {
    let fields = get_fields();
    let field_names = vec![
        "voidField",
        "boolField",
        "int8Field",
        "int16Field",
        "int32Field",
        "int64Field",
        "uInt8Field",
        "uInt16Field",
        "uInt32Field",
        "uInt64Field",
        "float32Field",
        "float64Field",
        "textField",
    ];
    for (i, field) in fields[0..field_names.len()].iter().enumerate() {
        assert_eq!(field.arrow_field().name, field_names[i]);
    }
}

#[test]
#[should_panic]
fn test_primitives_inner_fields() {
    let fields = get_fields();
    let field_names = vec![
        "voidField",
        "boolField",
        "int8Field",
        "int16Field",
        "int32Field",
        "int64Field",
        "uInt8Field",
        "uInt16Field",
        "uInt32Field",
        "uInt64Field",
        "float32Field",
        "float64Field",
        "textField",
    ];
    for field in fields[0..field_names.len()].iter() {
        let _ = field.inner_fields();
    }
}

#[test]
fn test_struct_field() {
    let fields = get_fields();
    assert_eq!(&fields[14].arrow_field().name, "structField");
    let children = &fields[14].inner_fields();
    assert_eq!(children.len(), 31);
    let void_field = &children[0];
    assert_eq!(void_field.arrow_field().name, "voidField");
    assert_eq!(
        void_field
            .capnp_field()
            .get_proto()
            .get_name()
            .unwrap()
            .to_string()
            .unwrap(),
        "voidField"
    );
}

#[test]
fn test_list_field() {
    let fields = get_fields();
    let children = &fields[16].inner_fields();
    assert_eq!(children.len(), 1);
    let inner_field = &fields[16].inner_field();
    assert_eq!(inner_field.arrow_field().name, "item");
    assert_eq!(
        fields[16]
            .capnp_field()
            .get_proto()
            .get_name()
            .unwrap()
            .to_string()
            .unwrap(),
        "voidList"
    );
}

#[test]
#[should_panic]
fn test_list_field_capnp_panic() {
    let fields = get_fields();
    let inner_field = &fields[16].inner_field();
    inner_field.capnp_field(); // No capnp equivalent field to list item, should panic
}

#[test]
#[should_panic]
fn test_struct_field_singular_inner_field_panic() {
    let fields = get_fields();
    let _ = &fields[14].inner_field(); // more than one inner field, should panic
}

#[test]
fn test_nested_struct_field() {
    let fields = get_fields();
    let struct0 = &fields[14].inner_fields();
    assert_eq!(&struct0[14].arrow_field().name, "structField");
    let struct1 = &struct0[14].inner_fields();
    assert_eq!(&struct1[14].arrow_field().name, "structField");
    let struct2 = &struct1[14].inner_fields();
    // We cut off `structField` to limit the depth of recursion
    assert_eq!(struct2[14].arrow_field().name, "enumField");
    assert_eq!(struct2.len(), 31);
    let primitive_child = &struct2[1];
    assert_eq!(primitive_child.arrow_field().name, "boolField");
    assert_eq!(
        primitive_child
            .capnp_field()
            .get_proto()
            .get_name()
            .unwrap()
            .to_string()
            .unwrap(),
        "boolField"
    );
    // The nested structs continue in the structList field
    let list_child = &struct2[29];
    assert_eq!(list_child.arrow_field().name, "structList");
    let list_struct_item0 = struct2[29].inner_field();
    let list_struct0 = list_struct_item0.inner_fields();
    assert_eq!(&list_struct0[0].arrow_field().name, "voidField");
}

#[test]
fn test_union_fields() {
    let fields: Vec<Field> = get_union_fields();
    assert_eq!(fields.len(), 4);
    assert_eq!(fields[0].arrow_field().name, "union0");
    assert_eq!(fields[1].arrow_field().name, "listOuter");
    assert_eq!(fields[2].arrow_field().name, "grault");
    assert_eq!(fields[3].arrow_field().name, "garply");
}

#[test]
fn test_nested_union_list_field() {
    let fields: Vec<Field> = get_union_fields();
    let list_children = &fields[1].inner_fields();
    assert_eq!(&list_children[0].arrow_field().name, "item");
    let outer_struct_children = &list_children[0].inner_fields();
    assert_eq!(&outer_struct_children[0].arrow_field().name, "union1");
    let union1_children = &outer_struct_children[0].inner_fields();
    assert_eq!(&union1_children[0].arrow_field().name, "listInner");
    let list_inner_children = &union1_children[0].inner_fields();
    assert_eq!(&list_inner_children[0].arrow_field().name, "item");
    let inner_struct_children = &list_inner_children[0].inner_fields();
    let baz = &inner_struct_children[0];
    assert_eq!(inner_struct_children.len(), 1);
    assert_eq!(baz.arrow_field().name, "baz");
    assert_eq!(
    baz.capnp_field()
        .get_proto()
        .get_name()
        .unwrap()
        .to_string()
        .unwrap(),
    "baz"
    );
}
