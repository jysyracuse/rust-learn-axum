use std;

fn print_type<T: Any>(value: &T) {
  println!("Type of value: {:?}", std::any::type_name::<T>());
}