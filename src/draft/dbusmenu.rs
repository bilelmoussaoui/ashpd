use std::collections::HashMap;

use serde::Serialize;
use zbus::{
    interface,
    object_server::SignalContext,
    zvariant::{Structure, StructureBuilder, Type, Value},
};

use crate::desktop::Icon;

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
    id: i32,
    properties: HashMap<String, Value<'static>>,
    children: Vec<Value<'static>>,
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

#[derive(Debug, Serialize, Type)]
pub(crate) struct DBusMenuUpdatedProperties {
    id: i32,
    properties: HashMap<String, Value<'static>>,
}

#[derive(Debug, Serialize, Type)]
pub(crate) struct DBusMenuRemovedProperties {
    id: i32,
    property_names: Vec<String>,
}

#[derive(Default)]
struct DBusMenuItemPrivate {
    id: i32,
    parent_id: i32,
    user_id: Option<String>,
    action: Option<Box<dyn Fn(String, Value) + Sync + Send>>,
    properties: HashMap<&'static str, Value<'static>>,
    children: Vec<i32>,
}

impl DBusMenuItemPrivate {
    fn filter_properties(&self, props: &Vec<String>) -> HashMap<String, Value<'static>> {
        if props.is_empty() {
            return self
                .properties
                .iter()
                .map(|(k, v)| (k.to_string(), v.try_clone().unwrap()))
                .collect();
        }
        let mut filtered_props = HashMap::default();
        for prop_name in props {
            let prop_name = prop_name.as_str();
            let Some(prop) = self.properties.get(prop_name) else {
                continue;
            };
            filtered_props.insert(prop_name.to_string(), prop.try_clone().unwrap());
        }
        filtered_props
    }

    pub(crate) fn take_properties(&mut self) -> Option<DBusMenuRemovedProperties> {
        if self.properties.is_empty() {
            return None;
        }
        let removed = DBusMenuRemovedProperties {
            id: self.id,
            property_names: self.properties.iter().map(|(k, _)| k.to_string()).collect(),
        };
        self.properties.clear();
        Some(removed)
    }
}

///
#[derive(Default)]
pub struct DBusMenuItem {
    user_id: Option<String>,
    action: Option<Box<dyn Fn(String, Value) + Sync + Send>>,
    properties: HashMap<&'static str, Value<'static>>,
    children: Vec<DBusMenuItem>,
}

impl DBusMenuItem {
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// `id` can be used to get a reference to this item later.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
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
    pub fn icon(mut self, icon: Icon) -> Self {
        match icon {
            Icon::Name(name) => {
                self.properties.insert("icon-name", Value::from(name));
            }
            Icon::Pixmaps(pixmaps) => {
                self.properties.insert("icon-data", Value::from(pixmaps));
            }
            _ => {}
        };
        self
    }

    ///
    pub fn on_click(mut self, on_click: Box<dyn Fn(String, Value) + Sync + Send>) -> Self {
        self.action = Some(on_click);
        self
    }

    ///
    pub fn children(mut self, mut children: Vec<DBusMenuItem>) -> Self {
        self.children.append(&mut children);
        self
    }
}
///
#[derive(Default)]
pub struct DBusMenu {
    next_id: i32,
    root: HashMap<i32, DBusMenuItemPrivate>,
}

impl DBusMenu {
    ///
    pub fn new() -> Self {
        Self {
            next_id: 1,
            root: HashMap::from_iter([(0, DBusMenuItemPrivate::default())]),
        }
    }

    ///
    pub fn submenu(mut self, submenu: DBusMenuItem) -> Self {
        self.add_to_root(submenu, 0);
        self
    }

    fn add_to_root(&mut self, submenu: DBusMenuItem, parent_id: i32) {
        let id = self.next_id;
        let result = DBusMenuItemPrivate {
            id,
            parent_id,
            user_id: submenu.user_id,
            action: submenu.action,
            properties: submenu.properties,
            ..Default::default()
        };
        self.next_id += 1;
        for child in submenu.children {
            self.add_to_root(child, id);
        }
        self.root.insert(id, result);
        if let Some(parent) = self.root.get_mut(&parent_id) {
            parent.children.push(id);
        }
    }

    fn to_dbus(&self, parent_id: i32, depth: i32, properties: &Vec<String>) -> DBusMenuLayoutItem {
        let parent = self.root.get(&parent_id).unwrap();
        let mut menu = DBusMenuLayoutItem {
            id: parent.id,
            ..Default::default()
        };
        menu.properties = parent.filter_properties(properties);
        if !parent.children.is_empty() && depth != 0 {
            menu.properties
                .insert("children-display".into(), Value::from("submenu"));
            for child_id in &parent.children {
                menu.children
                    .push(Value::from(self.to_dbus(*child_id, depth - 1, properties)));
            }
        }
        menu
    }
}

#[derive(Default)]
pub(crate) struct DBusMenuInterface {
    pub(crate) menu: DBusMenu,
    pub(crate) revision: u32,
}

#[interface(name = "com.canonical.dbusmenu")]
impl DBusMenuInterface {
    #[zbus(out_args("revision", "layout"))]
    async fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        properties: Vec<String>,
    ) -> (u32, DBusMenuLayoutItem) {
        let menu = self.menu.to_dbus(parent_id, recursion_depth, &properties);
        (self.revision, menu)
    }

    async fn get_group_properties(
        &self,
        ids: Vec<i32>,
        property_names: Vec<String>,
    ) -> Vec<DBusMenuUpdatedProperties> {
        let mut properties = Vec::default();
        for id in ids {
            let menu = self.menu.root.get(&id).unwrap();
            let new_properties = menu.filter_properties(&property_names);
            properties.push(DBusMenuUpdatedProperties {
                id: menu.id,
                properties: new_properties,
            });
        }
        properties
    }

    async fn event(&self, id: i32, event_id: String, event_data: Value<'_>, _timestamp: u32) {
        if event_id.eq("clicked") {
            let menu = self.menu.root.get(&id).unwrap();
            menu.action
                .as_ref()
                .map(|action| action(event_id, event_data));
        }
    }

    async fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    #[zbus(signal, name = "ItemsPropertiesUpdated")]
    pub(crate) async fn items_properties_updated(
        &self,
        cx: &SignalContext<'_>,
        updated: Vec<DBusMenuUpdatedProperties>,
        removed: Vec<DBusMenuRemovedProperties>,
    ) -> zbus::Result<()>;

    #[zbus(signal, name = "LayoutUpdated")]
    pub(crate) async fn layout_updated(
        &self,
        cx: &SignalContext<'_>,
        revision: u32,
        parent: i32,
    ) -> zbus::Result<()>;
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
        // let menu = DBusMenu::from_items(Vec::from_iter([
        //     DBusMenuItemPrivate::new().children(Vec::from_iter([
        //         DBusMenuItemPrivate::new(),
        //         DBusMenuItemPrivate::new(),
        //     ])),
        //     DBusMenuItemPrivate::new(),
        // ]));
        // let mut queue = VecDeque::new();
        // let mut set: HashSet<i32> = HashSet::new();
        // queue.push_back(menu.root);
        // while !queue.is_empty() {
        //     let item = queue.pop_front().unwrap();
        //     assert!(set.insert(item.id));
        //     for child in item.children {
        //         queue.push_back(child);
        //     }
        // }
    }

    #[test]
    fn test_dbus_user_id() {
        // let mut menu = DBusMenu::from_items(Vec::from_iter([
        //     DBusMenuItemPrivate::new()
        //         .id("id1")
        //         .label("Test1")
        //         .children(Vec::from_iter([
        //             DBusMenuItemPrivate::new().id("id10").label("Test-1"),
        //             DBusMenuItemPrivate::new().id("id20").label("Test-2"),
        //         ])),
        //     DBusMenuItemPrivate::new().id("id2").label("Test2"),
        // ]));
        // let mut found = menu.find_by_user_id_mut("id1");
        // assert!(found.is_some());

        // found = menu.find_by_user_id_mut("id2");
        // assert!(found.is_some());

        // found = menu.find_by_user_id_mut("id10");
        // assert!(found.is_some());

        // found = menu.find_by_user_id_mut("id20");
        // assert!(found.is_some());

        // found = menu.find_by_user_id_mut("id21");
        // assert!(found.is_none());
    }
}
