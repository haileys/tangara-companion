use glib::object::ObjectType;
use glib::WeakRef;

pub fn weak<T: ObjectType>(object: &T) -> WeakRef<T> {
    let ref_ = WeakRef::new();
    ref_.set(Some(object));
    ref_
}
