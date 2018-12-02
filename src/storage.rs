use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Storage {
    devices: HashSet<String>,
    notifications: HashMap<String, VecDeque<Event>>,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub event_type: EventType,
    pub message: Option<String>,
}

#[derive(Clone, Debug)]
pub enum EventType {
    SLAP,
    MESSAGE,
}

impl Event {
    pub fn new_slap() -> Event {
        Event {
            event_type: EventType::SLAP,
            message: None,
        }
    }

    pub fn new_message(text: String) -> Event {
        Event {
            event_type: EventType::MESSAGE,
            message: Some(text),
        }
    }
}

impl Storage {
    pub fn new() -> Storage {
        let mut storage = Storage {
            devices: HashSet::new(),
            notifications: HashMap::new(),
        };

        storage.devices.insert(String::from("MILAN"));
        storage.devices.insert(String::from("POLAND"));
        storage.devices.insert(String::from("KIEV"));
        storage.devices.insert(String::from("BERLIN"));

        for device in &storage.devices {
            let queue = VecDeque::new();
            storage.notifications.insert(device.clone(), queue);
        }

        storage
    }

    pub fn is_registered(&self, device: &String) -> bool {
        self.devices.contains(device)
    }

    pub fn get_supported_cities_as_str(&self) -> String {
        self.devices
            .clone()
            .into_iter()
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn add_event(&mut self, event: Event, to_device: String) {
        if self.devices.contains(&to_device) {
            match self.notifications.get_mut(&to_device) {
                Some(queue) => queue.push_back(event),
                _ => (),
            }
        }
    }

    pub fn pop_event(&mut self, for_device: &String) -> Option<Event> {
        match self.notifications.get_mut(for_device) {
            Some(queue) => queue.pop_front(),
            _ => None,
        }
    }

    pub fn size(&self, for_device: &String) -> usize {
        match self.notifications.get(for_device) {
            Some(queue) => queue.len(),
            _ => 0,
        }
    }
}

#[test]
fn smoke_test_storage() {
    let mut storage = Storage::new();

    let event = Event::new_slap();

    storage.add_event(event.clone(), String::from("MILAN"));

    match storage.pop_event(&String::from("MILAN")) {
        Some(_poped_event) => (),
        _ => panic!(),
    }
}

#[test]
fn smoke_test_empty_storage() {
    let mut storage = Storage::new();

    let event = Event::new_slap();

    storage.add_event(event.clone(), String::from("MILAN"));
    storage.add_event(event.clone(), String::from("MILAN"));
    println!("test debug {:?}", &storage);

    match storage.pop_event(&String::from("BERLIN")) {
        Some(_) => panic!(),
        _ => assert!(true),
    }

    match storage.pop_event(&String::from("MILAN")) {
        Some(_poped_event) => (),
        _ => panic!(),
    }

    match storage.pop_event(&String::from("MILAN")) {
        Some(_poped_event) => (),
        _ => panic!(),
    }

    match storage.pop_event(&String::from("MILAN")) {
        Some(_) => panic!(),
        _ => assert!(true),
    }
}

#[test]
fn test_size() {
    let mut storage = Storage::new();

    let event = Event::new_slap();

    storage.add_event(event.clone(), String::from("MILAN"));
    storage.add_event(event.clone(), String::from("MILAN"));

    assert_eq!(storage.size(&String::from("BERLIN")), 0);
    assert_eq!(storage.size(&String::from("MILAN")), 2);
}
