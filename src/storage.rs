use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::VecDeque;


pub struct Storage {
    devices: HashSet<String>,
    notifications: HashMap<String, VecDeque<Event>>
}

#[derive(Clone, Debug)]
pub struct Event {
    pub from_device: String,
    pub event_type: EventType
}

#[derive(Clone, Debug)]
pub enum EventType {
    SLAP
}

impl Storage {
    pub fn new () -> Storage {
        let mut storage = Storage {
            devices: HashSet::new(),
            notifications: HashMap::new()
        };

        storage.devices.insert(String::from("Milan"));
        storage.devices.insert(String::from("Poland"));
        storage.devices.insert(String::from("Kiev"));
        storage.devices.insert(String::from("Berlin"));

        for device in &storage.devices {
            let queue = VecDeque::new();
            storage.notifications.insert(device.clone(), queue);
        }

        storage
    }

    pub fn add_event(&mut self, event: Event, to_device: String) {
        if self.devices.contains(&to_device) {
            match self.notifications.get_mut(&to_device) {
                Some(queue) => queue.push_back(event),
                _ => ()
            }

        }
    }

    pub fn pop_event(&mut self, for_device: String) -> Option<Event> {
        match self.notifications.get_mut(&for_device) {
            Some(queue) => queue.pop_front(),
            _ => None
        }
    }

    pub fn size(&self, for_device: String) -> usize {
        match self.notifications.get(&for_device) {
            Some(queue) => queue.len(),
            _ => 0
        }
    }

}


#[test]
fn smoke_test_storage() {
    let mut storage = Storage::new();

    let event = Event {
        from_device: String::from("Berlin"),
        event_type: EventType::SLAP
    };

    storage.add_event(event.clone(), String::from("Milan"));

    match storage.pop_event(String::from("Milan")) {
        Some(poped_event) => assert_eq!(event.from_device, poped_event.from_device),
        _ => panic!()
    }

}

#[test]
fn smoke_test_empty_storage() {
    let mut storage = Storage::new();

    let event = Event {
        from_device: String::from("Berlin"),
        event_type: EventType::SLAP
    };

    storage.add_event(event.clone(), String::from("Milan"));
    storage.add_event(event.clone(), String::from("Milan"));

    match storage.pop_event(String::from("Berlin")) {
        Some(_) => panic!(),
        _ => assert!(true)
    }

    match storage.pop_event(String::from("Milan")) {
        Some(poped_event) => assert_eq!(event.from_device, poped_event.from_device),
        _ => panic!()
    }

    match storage.pop_event(String::from("Milan")) {
        Some(poped_event) => assert_eq!(event.from_device, poped_event.from_device),
        _ => panic!()
    }

    match storage.pop_event(String::from("Milan")) {
        Some(_) => panic!(),
        _ => assert!(true)
    }

}

#[test]
fn test_size() {
    let mut storage = Storage::new();

    let event = Event {
        from_device: String::from("Berlin"),
        event_type: EventType::SLAP
    };

    storage.add_event(event.clone(), String::from("Milan"));
    storage.add_event(event.clone(), String::from("Milan"));

    assert_eq!(storage.size(String::from("Berlin")), 0);
    assert_eq!(storage.size(String::from("Milan")), 2);
}

