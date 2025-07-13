# Meeting Cost Tracker

`meeting_cost_tracker` provides a small library and optional terminal user
interface for monitoring the real-time cost of meetings. It calculates cost
based on the salaries of attendees and the elapsed time of a meeting.

The library is intended for integration into other tools but also ships with a
TUI binary that demonstrates its capabilities.

## Example

```rust
use meeting_cost_tracker::{EmployeeCategory, Meeting};

let category = EmployeeCategory::new("Engineer", 120_000).unwrap();
let mut meeting = Meeting::new();
meeting.add_attendee(&category, 3);
meeting.start();
std::thread::sleep(std::time::Duration::from_secs(1));
meeting.stop();
println!("Cost: ${:.2}", meeting.total_cost());
```

Running the provided binary gives an interactive interface for adding employee
categories and tracking a meeting live:

```console
$ cargo run --release --bin mct
```

After installing the crate with `cargo install --path .`, you can run the TUI
directly using the `mct` command.

## See Also

- [`Meeting`](src/meeting.rs) – core meeting logic.
- [`EmployeeCategory`](src/model.rs) – employee salary representation.
- [`load_categories`](src/storage.rs) – persistence helpers.

