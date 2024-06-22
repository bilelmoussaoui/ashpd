use std::collections::HashMap;

use serde::Serialize;
use std::collections::VecDeque;
use zbus::{
    interface,
    object_server::SignalContext,
    zvariant::{OwnedValue, Structure, StructureBuilder, Type, Value},
};

///
pub const DBUS_MENU_PATH: &str = "/MenuBar";

///
#[derive(Clone)]
pub enum MenuType {
    /// "standard"
    Standard,
    /// "separator"
    Separator,
}

///
#[derive(Clone)]
pub enum MenuToggleType {
    /// "checkmark"
    Checkmark,
    /// "radio"
    Radio,
}

#[derive(Default, Serialize, Type)]
struct DBusMenuLayoutItem {
    pub id: i32,
    pub properties: HashMap<&'static str, OwnedValue>,
    pub children: Vec<Value<'static>>,
}

impl<'a> From<DBusMenuLayoutItem> for Structure<'a> {
    fn from(value: DBusMenuLayoutItem) -> Self {
        StructureBuilder::new()
            .add_field(value.id)
            .add_field(value.properties)
            .add_field(value.children)
            .build()
    }
}

///
#[derive(Default)]
pub struct DBusMenuItem {
    id: i32,
    action: Option<Box<dyn Fn(String, Value) + Sync + Send>>,
    properties: HashMap<&'static str, Value<'static>>,
    children: Vec<DBusMenuItem>,
}

impl DBusMenuItem {
    ///
    pub fn new() -> Self {
        Self::default()
    }

    ///
    pub fn menu_type(mut self, menu_type: MenuType) -> Self {
        match menu_type {
            MenuType::Standard => self.properties.remove("type"),
            MenuType::Separator => self.properties.insert("type", Value::from("separator")),
        };
        self
    }

    ///
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.properties.insert("label", Value::from(text.into()));
        self
    }

    ///
    pub fn enabled(mut self, enabled: bool) -> Self {
        match enabled {
            true => self.properties.remove("enabled"),
            false => self.properties.insert("enabled", Value::from(enabled)),
        };
        self
    }

    ///
    pub fn visible(mut self, visible: bool) -> Self {
        match visible {
            true => self.properties.remove("visible"),
            false => self.properties.insert("visible", Value::from(visible)),
        };
        self
    }

    ///
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.properties
            .insert("icon-name", Value::from(icon.into()));
        self
    }

    ///
    pub fn on_click(mut self, on_click: Box<dyn Fn(String, Value) + Sync + Send>) -> Self {
        self.action = Some(on_click);
        self
    }

    ///
    pub fn children(mut self, children: Vec<DBusMenuItem>) -> Self {
        self.children = children;
        self
    }

    fn fill_ids(&mut self, next_id: &mut i32) {
        self.id = *next_id;
        *next_id += 1;
        for child in &mut self.children {
            child.fill_ids(next_id);
        }
    }

    fn to_dbus(&self, depth: i32) -> DBusMenuLayoutItem {
        let mut menu = DBusMenuLayoutItem {
            id: self.id,
            ..Default::default()
        };
        if !self.children.is_empty() && depth != 0 {
            menu.properties.insert(
                "children-display".into(),
                Value::from("submenu").try_to_owned().unwrap(),
            );
            menu.properties = self
                .properties
                .iter()
                .map(|(k, v)| ((*k), v.try_to_owned().unwrap()))
                .collect();
            for child in &self.children {
                menu.children.push(Value::from(child.to_dbus(depth - 1)));
            }
        }
        menu
    }
}

///
#[derive(Default)]
pub struct DBusMenu {
    next_id: i32,
    root: DBusMenuItem,
}

impl DBusMenu {
    ///
    pub fn from_items(items: Vec<DBusMenuItem>) -> DBusMenu {
        let mut menu = Self {
            next_id: 1,
            root: DBusMenuItem::default(),
        };
        for mut item in items.into_iter() {
            item.fill_ids(&mut menu.next_id);
            menu.root.children.push(item);
        }
        menu
    }
}

///
#[derive(Default)]
pub struct DBusMenuInterface {
    pub(crate) menu: DBusMenu,
    pub(crate) revision: u32,
}

///
#[interface(name = "com.canonical.dbusmenu")]
impl DBusMenuInterface {
    #[zbus(out_args("revision", "layout"))]
    async fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        _property_names: Vec<String>,
    ) -> (u32, DBusMenuLayoutItem) {
        let mut main_menu = DBusMenuLayoutItem::default();
        let menu = self.find_by_id(parent_id).unwrap();
        if !menu.children.is_empty() {
            main_menu.properties.insert(
                "children-display".into(),
                Value::from("submenu").try_to_owned().unwrap(),
            );
            for child in &menu.children {
                main_menu
                    .children
                    .push(Value::from(child.to_dbus(recursion_depth)));
            }
        }
        (self.revision, main_menu)
    }

    ///
    async fn event(&self, id: i32, event_id: String, event_data: Value<'_>, _timestamp: u32) {
        let menu = self.find_by_id(id).unwrap();
        menu.action
            .as_ref()
            .map(|action| action(event_id, event_data));
    }

    /// TODO: Not sure what is the purpose of this.
    async fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    ///
    #[zbus(signal, name = "ItemsPropertiesUpdated")]
    pub async fn items_properties_updated(
        &self,
        cx: &SignalContext<'_>,
        properties: Vec<(i32, HashMap<String, OwnedValue>)>,
    ) -> zbus::Result<()>;

    ///
    #[zbus(signal, name = "LayoutUpdated")]
    pub async fn layout_updated(
        &self,
        cx: &SignalContext<'_>,
        revision: u32,
        parent: i32,
    ) -> zbus::Result<()>;
}

impl DBusMenuInterface {
    fn find_by_id(&self, id: i32) -> Option<&DBusMenuItem> {
        let mut result: Option<&DBusMenuItem> = None;
        let mut queue: VecDeque<&DBusMenuItem> = VecDeque::default();
        queue.push_back(&self.menu.root);
        while !queue.is_empty() {
            let menu = queue.pop_front().unwrap();
            if menu.id == id {
                result = Some(menu);
                break;
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_dbusmenu_signature() {
        let signature = DBusMenuLayoutItem::signature();
        assert_eq!(signature.as_str(), "(ia{sv}av)");
    }

    #[test]
    fn test_dbusmenu_unique_ids() {
        let menu = DBusMenu::from_items(Vec::from_iter([
            DBusMenuItem::new()
                .children(Vec::from_iter([DBusMenuItem::new(), DBusMenuItem::new()])),
            DBusMenuItem::new(),
        ]));
        let mut queue = VecDeque::new();
        let mut set: HashSet<i32> = HashSet::new();
        queue.push_back(menu.root);
        while !queue.is_empty() {
            let item = queue.pop_front().unwrap();
            assert!(set.insert(item.id));
            for child in item.children {
                queue.push_back(child);
            }
        }
    }
}
