
#[cfg(windows)]
#[allow(dead_code)]
mod windows {
    use std::io::{self, Write};

    type Handle = *mut std::ffi::c_void;

    const STD_INPUT_HANDLE: u32 = (-10i32) as u32;
    const STD_OUTPUT_HANDLE: u32 = (-11i32) as u32;

    const ENABLE_ECHO_INPUT: u32 = 0x0004;
    const ENABLE_LINE_INPUT: u32 = 0x0002;
    const ENABLE_PROCESSED_INPUT: u32 = 0x0001;

    const KEY_EVENT: u16 = 0x0001;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Coord {
        x: i16,
        y: i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SmallRect {
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct ConsoleScreenBufferInfo {
        size: Coord,
        cursor_position: Coord,
        attributes: u16,
        window: SmallRect,
        max_window_size: Coord,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    union KeyEventChar {
        unicode_char: u16,
        ascii_char: u8,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct KeyEventRecord {
        key_down: i32,
        repeat_count: u16,
        virtual_key_code: u16,
        virtual_scan_code: u16,
        unicode_char: KeyEventChar,
        control_key_state: u32,
    }

    #[repr(C)]
    union InputRecordData {
        key_event: KeyEventRecord,
    }

    #[repr(C)]
    struct InputRecord {
        event_type: u16,
        data: InputRecordData,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(n_std_handle: u32) -> Handle;

        fn GetConsoleMode(
            console_handle: Handle,
            mode: *mut u32,
        ) -> i32;

        fn SetConsoleMode(
            console_handle: Handle,
            mode: u32,
        ) -> i32;

        fn ReadConsoleInputW(
            console_handle: Handle,
            buffer: *mut InputRecord,
            length: u32,
            number_read: *mut u32,
        ) -> i32;

        fn GetConsoleScreenBufferInfo(
            console_handle: Handle,
            info: *mut ConsoleScreenBufferInfo,
        ) -> i32;
    }

    #[derive(Debug, Clone, Copy)]
    enum Key {
        Character(char),
        Enter,
        Backspace,
        Delete,
        Left,
        Right,
        Home,
        End,
        Up,
        Down,
        CtrlC,
        CtrlD,
    }

    fn read_key(handle: Handle) -> io::Result<Key> {
        loop {
            let mut record = std::mem::MaybeUninit::<InputRecord>::uninit();
            let mut read = 0u32;

            let result = unsafe {
                ReadConsoleInputW(
                    handle,
                    record.as_mut_ptr(),
                    1,
                    &mut read,
                )
            };

            if result == 0 {
                return Err(io::Error::last_os_error());
            }

            if read == 0 {
                continue;
            }

            let record = unsafe { record.assume_init() };

            if record.event_type != KEY_EVENT {
                continue;
            }

            let key = unsafe { record.data.key_event };

            if key.key_down == 0 {
                continue;
            }

            let ctrl = key.control_key_state & 0x0002 != 0
                || key.control_key_state & 0x0008 != 0;

            if ctrl && key.virtual_key_code == 0x43 {
                return Ok(Key::CtrlC);
            }

            if ctrl && key.virtual_key_code == 0x44 {
                return Ok(Key::CtrlD);
            }

            if let 0x0D = key.virtual_key_code {
                return Ok(Key::Enter);
            } else if let 0x08 = key.virtual_key_code {
                return Ok(Key::Backspace);
            } else if let 0x2E = key.virtual_key_code {
                return Ok(Key::Delete);
            } else if let 0x25 = key.virtual_key_code {
                return Ok(Key::Left);
            } else if let 0x27 = key.virtual_key_code {
                return Ok(Key::Right);
            } else if let 0x24 = key.virtual_key_code {
                return Ok(Key::Home);
            } else if let 0x23 = key.virtual_key_code {
                return Ok(Key::End);
            } else if let 0x26 = key.virtual_key_code {
                return Ok(Key::Up);
            } else if let 0x28 = key.virtual_key_code {
                return Ok(Key::Down);
            } else {
                let c = unsafe { key.unicode_char.unicode_char };

                if c == 0 {
                    continue;
                }

                return Ok(Key::Character(
                    char::from_u32(c as u32).unwrap_or('\0'),
                ));
            }
        }
    }

    fn redraw(prompt: &str, buffer: &str, cursor: usize) -> io::Result<()> {
        print!("\r{}{}", prompt, buffer);
        print!("\x1b[K");

        let distance = buffer.chars().count().saturating_sub(cursor);

        if distance > 0 {
            print!("\x1b[{}D", distance);
        }

        io::stdout().flush()
    }

    pub fn read_line(
        prompt: &str,
        history: &[String],
    ) -> io::Result<Option<String>> {
        let input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
        let _output = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };

        if input.is_null() {
            return Ok(None);
        }

        let mut original_mode = 0u32;

        let result = unsafe {
            GetConsoleMode(input, &mut original_mode)
        };

        if result == 0 {
            return Ok(None);
        }

        let new_mode = original_mode
            & !(ENABLE_LINE_INPUT
                | ENABLE_ECHO_INPUT
                | ENABLE_PROCESSED_INPUT);

        let result = unsafe {
            SetConsoleMode(input, new_mode)
        };

        if result == 0 {
            return Ok(None);
        }

        struct RestoreMode {
            handle: Handle,
            mode: u32,
        }

        impl Drop for RestoreMode {
            fn drop(&mut self) {
                unsafe {
                    SetConsoleMode(self.handle, self.mode);
                }
            }
        }

        let _restore = RestoreMode {
            handle: input,
            mode: original_mode,
        };

        print!("{prompt}");
        io::stdout().flush()?;

        let mut buffer = String::new();
        let mut cursor = 0usize;
        let mut history_index = history.len();

        loop {
            match read_key(input)? {
                Key::Character(c) => {
                    buffer.insert(cursor, c);
                    cursor += c.len_utf8();

                    redraw(prompt, &buffer, buffer.chars().count())?;

                    let remaining = buffer.chars().count() - cursor;
                    if remaining > 0 {
                        print!("\x1b[{}D", remaining);
                        io::stdout().flush()?;
                    }
                }

                Key::Enter => {
                    println!();
                    return Ok(Some(buffer));
                }

                Key::Backspace => {
                    if cursor > 0 {
                        let previous = buffer[..cursor]
                            .chars()
                            .next_back()
                            .map(char::len_utf8)
                            .unwrap_or(0);

                        buffer.drain(cursor - previous..cursor);
                        cursor -= previous;

                        redraw(
                            prompt,
                            &buffer,
                            buffer[..cursor].chars().count(),
                        )?;
                    }
                }

                Key::Delete => {
                    if cursor < buffer.len() {
                        let next = buffer[cursor..]
                            .chars()
                            .next()
                            .map(char::len_utf8)
                            .unwrap_or(0);

                        buffer.drain(cursor..cursor + next);

                        redraw(
                            prompt,
                            &buffer,
                            buffer[..cursor].chars().count(),
                        )?;
                    }
                }

                Key::Left => {
                    if cursor > 0 {
                        let previous = buffer[..cursor]
                            .chars()
                            .next_back()
                            .map(char::len_utf8)
                            .unwrap_or(0);

                        cursor -= previous;
                        print!("\x1b[1D");
                        io::stdout().flush()?;
                    }
                }

                Key::Right => {
                    if cursor < buffer.len() {
                        let next = buffer[cursor..]
                            .chars()
                            .next()
                            .map(char::len_utf8)
                            .unwrap_or(0);

                        cursor += next;
                        print!("\x1b[1C");
                        io::stdout().flush()?;
                    }
                }

                Key::Home => {
                    let position = buffer[..cursor].chars().count();

                    if position > 0 {
                        print!("\x1b[{}D", position);
                        io::stdout().flush()?;
                    }

                    cursor = 0;
                }

                Key::End => {
                    let current = buffer[..cursor].chars().count();
                    let target = buffer.chars().count();

                    if target > current {
                        print!("\x1b[{}C", target - current);
                        io::stdout().flush()?;
                    }

                    cursor = buffer.len();
                }

                Key::Up => {
                    if history.is_empty() {
                        continue;
                    }

                    if history_index > 0 {
                        history_index -= 1;
                        buffer = history[history_index].clone();
                        cursor = buffer.len();

                        redraw(prompt, &buffer, buffer.chars().count())?;
                    }
                }

                Key::Down => {
                    if history.is_empty() {
                        continue;
                    }

                    if history_index + 1 < history.len() {
                        history_index += 1;
                        buffer = history[history_index].clone();
                    } else {
                        history_index = history.len();
                        buffer.clear();
                    }

                    cursor = buffer.len();

                    redraw(prompt, &buffer, buffer.chars().count())?;
                }

                Key::CtrlC => {
                    println!("^C");
                    return Ok(Some(String::new()));
                }

                Key::CtrlD => {
                    if buffer.is_empty() {
                        println!();
                        return Ok(None);
                    }
                }
            }
        }
    }
}

#[cfg(not(windows))]
pub fn read_line(
    prompt: &str,
    _history: &[String],
) -> io::Result<Option<String>> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();

    match io::stdin().read_line(&mut input)? {
        0 => Ok(None),
        _ => Ok(Some(
            input.trim_end_matches(['\r', '\n']).to_string(),
        )),
    }
}