#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use meeting_cost_tracker::{load_categories, save_categories, EmployeeCategory, Meeting};

    #[test]
    fn test_employee_category_creation() {
        let valid = EmployeeCategory::new("Engineer", 100_000.0);
        assert!(valid.is_ok());

        let empty_title = EmployeeCategory::new("", 100_000.0);
        assert!(empty_title.is_err());

        let zero_salary = EmployeeCategory::new("Manager", 0.0);
        assert!(zero_salary.is_err());
    }

    #[test]
    fn test_meeting_cost_accumulation() {
        let cat = EmployeeCategory::new("Dev", 120_000.0).unwrap();
        let mut meeting = Meeting::new();
        meeting.add_attendee(&cat, 2);
        meeting.start();
        std::thread::sleep(Duration::from_millis(50));
        meeting.stop();

        let cost = meeting.total_cost();
        assert!(cost > 0.0);
    }

    #[test]
    fn test_meeting_reset() {
        let cat = EmployeeCategory::new("Analyst", 90_000.0).unwrap();
        let mut meeting = Meeting::new();
        meeting.add_attendee(&cat, 1);
        meeting.start();
        std::thread::sleep(Duration::from_millis(20));
        meeting.stop();
        meeting.reset();

        assert_eq!(meeting.total_cost(), 0.0);
    }

    #[test]
    fn test_add_and_remove_attendees() {
        let cat = EmployeeCategory::new("QA", 80_000.0).unwrap();
        let mut meeting = Meeting::new();
        meeting.add_attendee(&cat, 3);
        assert_eq!(meeting.attendees().next().unwrap().1 .1, 3);
        meeting.remove_attendee(cat.title(), 2);
        assert_eq!(meeting.attendees().next().unwrap().1 .1, 1);
        meeting.remove_attendee(cat.title(), 1);
        assert!(meeting.attendees().next().is_none());
    }

    #[test]
    fn test_persistence_round_trip() {
        let path = PathBuf::from("test_categories.toml");
        let original = vec![
            EmployeeCategory::new("A", 10_000.0).unwrap(),
            EmployeeCategory::new("B", 20_000.0).unwrap(),
        ];
        save_categories(&path, &original).unwrap();
        let loaded = load_categories(&path).unwrap();
        assert_eq!(original, loaded);
        std::fs::remove_file(&path).unwrap();
    }
}
