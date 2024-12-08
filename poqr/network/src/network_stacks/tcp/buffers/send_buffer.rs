use std::cmp::min;
use std::time::{SystemTime, UNIX_EPOCH};

const SEND_BUFFER_SIZE: usize = 65535;

/// A TCP socket's send buffer.
pub struct TcpSendBuffer {
    /// Data contained in the send buffer
    pub data: [u8; 65535],
    /// Start index of the circular buffer (inclusive; index of SND.UNA in the buffer)
    pub start_index: usize,
    /// End index of the circular buffer (exclusive)
    pub end_index: usize,
    /// Whether the buffer is full (needed to handle the case where the end wraps around)
    pub is_full: bool,
    /// (SND.WND) Size of the sliding window (advertised by the receiver)
    pub window_size: Option<u16>,
    /// (SND.NXT) Next sequence number to be sent
    pub next: u32,
    /// (SND.UNA) Lowest unacknowledged sequence number sent
    pub unacknowledged: u32,
    /// (ISS) Initial send sequence number for the socket connection
    pub initial_sequence_number: u32,
}

impl TcpSendBuffer {
    /// Construct a new `TcpSendBuffer`.
    pub fn new() -> TcpSendBuffer {
        let initial_sequence_number = TcpSendBuffer::generate_initial_sequence_number();

        TcpSendBuffer {
            data: [0; SEND_BUFFER_SIZE],
            start_index: 0,
            end_index: 0,
            is_full: false,
            window_size: None,
            next: initial_sequence_number,
            unacknowledged: initial_sequence_number,
            initial_sequence_number,
        }
    }

    /// Get the index in the circular buffer corresponding to the next sequence number to be sent (`SND.NXT`).
    pub fn next_index(&self) -> usize {
        (self.start_index + self.num_bytes_in_flight() as usize) % self.data.len()
    }

    /// Get the index in the circular buffer corresponding to the end of the send window.
    pub fn window_end_index(&self) -> usize {
        (self.start_index + self.window_size.unwrap_or(0) as usize) % self.data.len()
    }

    /// Increment the `start_index` (i.e. index of `SND.UNA`) by the given amount.
    pub fn increment_start_index(&mut self, n: usize) {
        self.start_index = (self.start_index + n) % self.data.len();
    }

    /// Increment the `end_index` by the given amount.
    pub fn increment_end_index(&mut self, n: usize) {
        self.end_index = (self.end_index + n) % self.data.len();
    }

    /// Increment the `next` pointer (`SND.NXT`) by the given amount.
    pub fn increment_next(&mut self, n: u32) {
        self.next = self.next.wrapping_add(n);
    }

    /// Increment the `unacknowledged` pointer (`SND.UNA`) by the given amount. Also adjusts the `start_index` of the circular buffer.
    pub fn increment_unacknowledged(&mut self, n: u32) {
        self.unacknowledged = self.unacknowledged.wrapping_add(n);
    }

    /// Generate an initial sequence number for the socket connection based on 4-microsecond ticks since the Unix epoch.
    pub fn generate_initial_sequence_number() -> u32 {
        let duration_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let microseconds_since_epoch = duration_since_epoch.as_micros();
        let initial_sequence_number = (microseconds_since_epoch / 4) as u32;
        initial_sequence_number
    }

    /// Get the number of bytes "in flight" (i.e. the number of bytes that have been sent but not yet acknowledged by the receiver).
    /// This should never exceed the receiver's advertised window size stored in `self.window_size`.
    pub fn num_bytes_in_flight(&self) -> u32 {
        self.next.wrapping_sub(self.unacknowledged)
    }

    /// Get the number of empty bytes in the circular buffer that can be written to.
    pub fn available_bytes(&self) -> usize {
        if self.is_full {
            // Buffer is full (need this check to disambiguate the two cases where start_index == end_index)
            0
        } else if self.end_index >= self.start_index {
            // SEND_BUFFER_SIZE - [start_index, end_index)
            self.data.len() - (self.end_index - self.start_index)
        } else {
            // SEND_BUFFER_SIZE - [start_index, SEND_BUFFER_SIZE-1] - [0, end_index)
            self.start_index - self.end_index
        }
    }

    /// Get the number of occupied bytes in the circular buffer.
    pub fn occupied_bytes(&self) -> usize {
        self.data.len() - self.available_bytes()
    }

    /// Get the number of sendable bytes in the circular buffer that are within the receiver's advertised window size
    /// (i.e. bytes between `SND.NXT` and `SND.UNA + SND.WND`).
    pub fn unsent_window_bytes(&self) -> usize {
        let end_index = if self.occupied_bytes() <= self.window_size.unwrap_or(0) as usize {
            // If the buffer is smaller than the window size, we can send everything
            self.end_index
        } else {
            // Otherwise, we can only send up to the window end index
            self.window_end_index()
        };

        if self.is_full && self.next_index() == end_index {
            // Buffer is full (need this check to disambiguate the two cases where next_index == window_end_index)
            0
        } else if end_index >= self.next_index() {
            // [next_index, end_index)
            end_index - self.next_index()
        } else {
            // [0, end_index) + [next_index, SEND_BUFFER_SIZE-1]
            end_index + self.data.len() - self.next_index()
        }
    }

    /// Push a slice onto the end of the send buffer. Returns the number of bytes written, which may be less
    /// than the length of the slice if the buffer reaches capacity.
    pub fn push(&mut self, buffer: &[u8]) -> usize {
        let bytes_to_write = min(self.available_bytes(), buffer.len());

        for i in 0..bytes_to_write {
            self.data[self.end_index] = buffer[i];
            self.increment_end_index(1);
        }

        // If the end index has caught up to the start index, the buffer is full
        if self.end_index == self.start_index {
            self.is_full = true;
        }

        bytes_to_write
    }

    /// Acknowledge the receipt of all bytes up to the given acknowledgment number and update the window size.
    /// Returns `Some(num_bytes_acknowledged)` if the acknowledgment advances the send window, and `None` otherwise.
    pub fn acknowledge(&mut self, acknowledgment_number: u32, window_size: u16) -> Option<u16> {
        let n = acknowledgment_number.wrapping_sub(self.unacknowledged);

        // Check if the acknowledgment falls between unacknowledged (SND.UNA) and next (SND.NXT)
        if n == 0 || n > self.num_bytes_in_flight() {
            return None;
        }

        self.window_size = Some(window_size);
        Some(n as u16)
    }

    /// Slide the send window by the given amount after receiving new ACKs, adjusting `SND.UNA` and `start_index` accordingly.
    /// Since acknowledged bytes can be overwritten, this will also set the buffer to not full.
    pub fn slide_window(&mut self, n: u16) {
        // Update the unacknowledged pointer and start index, and set the buffer to not full
        self.increment_start_index(n as usize);
        self.increment_unacknowledged(n as u32);
        self.is_full = false;
    }
}
