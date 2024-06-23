use std::collections::HashMap;

use serde::Serialize;
use std::collections::VecDeque;
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
struct DBusMenuLayoutProperties {
    id: i32,
    properties: HashMap<String, Value<'static>>,
}

///
#[derive(Default)]
pub struct DBusMenuItem {
    id: i32,
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
    pub fn child(mut self, child: DBusMenuItem) -> Self {
        self.children.push(child);
        self
    }

    ///
    pub fn children(mut self, mut children: Vec<DBusMenuItem>) -> Self {
        self.children.append(&mut children);
        self
    }

    fn fill_ids(&mut self, next_id: &mut i32) {
        self.id = *next_id;
        *next_id += 1;
        self.children.iter_mut().for_each(|child| {
            child.fill_ids(next_id);
        });
    }

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

    fn to_dbus(&self, depth: i32, properties: &Vec<String>) -> DBusMenuLayoutItem {
        let mut menu = DBusMenuLayoutItem {
            id: self.id,
            ..Default::default()
        };
        menu.properties = self.filter_properties(properties);
        if !self.children.is_empty() && depth != 0 {
            menu.properties
                .insert("children-display".into(), Value::from("submenu"));

            for child in &self.children {
                menu.children
                    .push(Value::from(child.to_dbus(depth - 1, properties)));
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

    /// Returns the id of the updated item
    pub(crate) fn update<F>(&mut self, user_id: &str, fun: F) -> Option<i32>
    where
        F: FnOnce(&mut DBusMenuItem),
    {
        let mut next_id = self.next_id;
        let Some(menu) =
            self.find_mut(|item| item.user_id.as_ref().map_or(false, |id| id.eq(user_id)))
        else {
            return None;
        };
        let updated = menu.id;
        fun(menu);
        for child in &mut menu.children {
            child.fill_ids(&mut next_id);
        }
        Some(updated)
    }

    fn find_by_id(&self, id: i32) -> Option<&DBusMenuItem> {
        self.find(|item| item.id == id)
    }

    fn find<F>(&self, mut compare: F) -> Option<&DBusMenuItem>
    where
        F: FnMut(&DBusMenuItem) -> bool,
    {
        let mut result: Option<&DBusMenuItem> = None;
        let mut queue: VecDeque<&DBusMenuItem> = VecDeque::default();
        queue.push_back(&self.root);
        while !queue.is_empty() {
            let menu = queue.pop_front().unwrap();
            if compare(menu) {
                result = Some(menu);
                break;
            }
            for child in &menu.children {
                queue.push_back(child);
            }
        }
        result
    }

    fn find_mut<F>(&mut self, mut compare: F) -> Option<&mut DBusMenuItem>
    where
        F: FnMut(&DBusMenuItem) -> bool,
    {
        let mut result: Option<&mut DBusMenuItem> = None;
        let mut queue: VecDeque<&mut DBusMenuItem> = VecDeque::default();
        queue.push_back(&mut self.root);
        while !queue.is_empty() {
            let menu = queue.pop_front().unwrap();
            if compare(menu) {
                result = Some(menu);
                break;
            }
            for child in &mut menu.children {
                queue.push_back(child);
            }
        }
        result
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
        let menu = self
            .menu
            .find_by_id(parent_id)
            .unwrap()
            .to_dbus(recursion_depth, &properties);
        println!("{parent_id} {recursion_depth}");
        (self.revision, menu)
    }

    async fn get_group_properties(
        &self,
        ids: Vec<i32>,
        property_names: Vec<String>,
    ) -> Vec<DBusMenuLayoutProperties> {
        let mut properties = Vec::default();
        for id in ids {
            let menu = self.menu.find_by_id(id).unwrap();
            let new_properties = menu.filter_properties(&property_names);
            properties.push(DBusMenuLayoutProperties {
                id: menu.id,
                properties: new_properties,
            });
        }
        properties
    }

    async fn event(&self, id: i32, event_id: String, event_data: Value<'_>, _timestamp: u32) {
        if event_id.eq("clicked") {
            let menu = self.menu.find_by_id(id).unwrap();
            menu.action
                .as_ref()
                .map(|action| action(event_id, event_data));
        }
    }

    async fn about_to_show(&self, _id: i32) -> bool {
        false
    }

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

    #[test]
    fn test_dbus_user_id() {
        let mut menu = DBusMenu::from_items(Vec::from_iter([
            DBusMenuItem::new()
                .id("id1")
                .label("Test1")
                .children(Vec::from_iter([
                    DBusMenuItem::new().id("id10").label("Test-1"),
                    DBusMenuItem::new().id("id20").label("Test-2"),
                ])),
            DBusMenuItem::new().id("id2").label("Test2"),
        ]));
        let mut found = menu.find_by_user_id_mut("id1");
        assert!(found.is_some());

        found = menu.find_by_user_id_mut("id2");
        assert!(found.is_some());

        found = menu.find_by_user_id_mut("id10");
        assert!(found.is_some());

        found = menu.find_by_user_id_mut("id20");
        assert!(found.is_some());

        found = menu.find_by_user_id_mut("id21");
        assert!(found.is_none());
    }
}
