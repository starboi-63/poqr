use std::cmp::{max, min};
use std::i64;

const RECEIVE_BUFFER_SIZE: usize = 65535;
pub const RECEIVE_WINDOW_SIZE: u16 = 32768;

/// A TCP socket's receive buffer.
pub struct TcpReceiveBuffer {
    /// Values are `None` if the byte has not been received yet
    pub data: [Option<u8>; RECEIVE_BUFFER_SIZE],
    /// Start index of the circular buffer (inclusive; index of next byte to read)
    pub start_index: usize,
    /// Index of self.next (RCV.NXT) in the buffer
    pub next_index: usize,
    /// Whether the buffer is full (needed to handle the case where the end wraps around)
    pub is_full: bool,
    /// (RCV.NXT) Sequence number of the next contiguous byte expected to be received
    pub next: u32,
    /// (RCV.WND) Advertised size of the sliding window (equal to end_index - start_index)
    pub window_size: u16,
    /// (IRS) Initial receive sequence number for the socket connection
    pub initial_sequence_number: u32,
}

impl TcpReceiveBuffer {
    /// Construct a new `TcpReceiveBuffer`.
    pub fn new(remote_sequence_number: u32) -> TcpReceiveBuffer {
        TcpReceiveBuffer {
            data: [None; RECEIVE_BUFFER_SIZE],
            start_index: 0,
            next_index: 0,
            is_full: false,
            next: remote_sequence_number,
            window_size: RECEIVE_WINDOW_SIZE,
            initial_sequence_number: remote_sequence_number,
        }
    }

    /// Set the next sequence number expected to be received (RCV.NXT).
    pub fn set_next(&mut self, value: u32) {
        self.next = value;
    }

    /// Increment the start index (i.e. last byte read pointer) by the given amount.
    pub fn increment_start_index(&mut self, n: usize) {
        self.start_index = (self.start_index + n) % self.data.len();
    }

    /// Increment the `next` pointer by the given amount.
    pub fn increment_next(&mut self, n: u32) {
        self.next = self.next.wrapping_add(n);
    }

    /// Increment the next index (i.e. next byte expected to be received pointer) by the given amount.
    fn increment_next_index(&mut self, n: usize) {
        self.next_index = (self.next_index + n) % self.data.len();
    }

    /// Returns the number of bytes that can be read from the buffer (equal to next_index -> start_index).
    pub fn readable_bytes(&self) -> usize {
        if self.is_full {
            self.data.len()
        } else if self.next_index >= self.start_index {
            self.next_index - self.start_index
        } else {
            self.next_index + self.data.len() - self.start_index
        }
    }

    /// Validate an incoming segment based on the four-case table in RFC 9293 3.4 (with mods for zero-window probing).
    /// Returns `true` if the segment is valid and should be processed, and `false` otherwise.
    pub fn is_valid_segment(&self, sequence_number: u32, data: &[u8]) -> bool {
        /// Helper function to check if a value is between two other values (inclusive with wraparound)
        fn is_between(start: u32, end: u32, value: u32) -> bool {
            if start <= end {
                value >= start && value <= end
            } else {
                value >= start || value <= end
            }
        }

        // Calculate the window and segment bounds
        let window_start = self.next;
        let window_end = self
            .next
            .wrapping_add(self.window_size as u32)
            .wrapping_sub(1);
        let segment_start = sequence_number;
        let segment_end = sequence_number
            .wrapping_add(data.len() as u32)
            .wrapping_sub(1);

        if data.len() == 0 && self.window_size == 0 {
            // If the window size is zero, we only accept segments with the expected sequence number
            sequence_number == self.next
        } else if data.len() == 0 && self.window_size > 0 {
            // If there is no data, we only accept segments within the window
            is_between(window_start, window_end, segment_start)
        } else if data.len() > 0 && self.window_size > 0 {
            // If there is data, we accept segments that overlap with the window
            is_between(window_start, window_end, segment_start)
                || is_between(window_start, window_end, segment_end)
        } else {
            // Otherwise, we reject the segment
            false
        }
    }

    /// Receive a valid data segment starting with the given sequence number. Returns `Some(num_contiguous_bytes)` if the
    /// segment advances the receive window, and `None` otherwise.
    pub fn receive_segment(&mut self, sequence_number: u32, data: &[u8]) -> Option<u16> {
        // Calculate the index in the buffer where the data should be written
        let offset: i64 = sequence_number as i64 - self.next as i64;

        // Write the data that overlaps with the sliding window
        let overlap: usize;

        if offset <= 0 {
            // Segment starts at or before the next pointer
            overlap = min(max(data.len() as i64 + offset, 0), self.window_size as i64) as usize;

            for i in (-offset as usize)..overlap {
                self.data[(self.next_index + i) % self.data.len()] = Some(data[i]);
            }
        } else {
            // Segment starts after the next pointer
            overlap = max(min(data.len() as i64, self.window_size as i64 - offset), 0) as usize;

            for i in 0..overlap {
                self.data[(offset as usize + self.next_index + i) % self.data.len()] =
                    Some(data[i]);
            }
        }

        // If sequence number == RCV.NXT, we've received the next expected segment and can slide the window
        if sequence_number == self.next {
            let mut num_contiguous_bytes = overlap;

            // Check if there are more contiguous bytes after the newly received segment
            while self.data[(self.next_index + num_contiguous_bytes) % self.data.len()].is_some()
                && num_contiguous_bytes < self.window_size as usize
            {
                num_contiguous_bytes += 1;
            }

            if num_contiguous_bytes > 0 {
                return Some(num_contiguous_bytes as u16);
            }
        }

        None
    }

    /// Slide the receive window by the given amount, adjusting the `next` pointer (RCV.NXT) and `window_size` (RCV.WIN) accordingly.
    pub fn slide_window(&mut self, n: u16) {
        let mut next_index_copy = self.next_index;
        self.increment_next(n as u32);
        self.increment_next_index(n as usize);

        // If the window hits the end of the buffer, shrink the window size
        for _ in 0..n {
            if (next_index_copy + self.window_size as usize) % self.data.len() == self.start_index {
                self.window_size -= 1;
            }
            next_index_copy = (next_index_copy + 1) % self.data.len();
        }

        // Check if the window size is zero; this implies that the buffer is full and
        // no more data can be received until some is read out of the buffer
        if self.window_size == 0 {
            self.is_full = true;
        }
    }
}
