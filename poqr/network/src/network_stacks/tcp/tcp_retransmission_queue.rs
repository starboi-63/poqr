use super::{TcpConnection, TcpNormalSocket, TcpPacket};
use crate::network_stacks::tcp::TcpConnectionState;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// RTT -> Round Trip Time
// RTO -> Retransmission Time Out
// SRTT -> Smoothed Round Trip Time

/// Smoothing factor for SRTT
const ALPHA: f32 = 0.8;
/// Delay variance factor for RTO
const BETA: f32 = 1.5;
/// Minimum RTO in seconds
const RTO_MIN: f32 = 0.1;
/// Maximum RTO in seconds
const RTO_MAX: f32 = 5.0;
/// Maximum number of retransmissions before giving up
const MAX_RETRANSMISSIONS: u8 = 5;

#[derive(Debug)]
/// The metadata associated with a segment in the retransmission queue
pub struct RetransmissionSegment {
    /// Byte data of the segment
    pub packet: TcpPacket,
    /// Time when the segment was sent
    pub send_time: Instant,
    /// Retransmission Timeout (RTO)
    pub rto: f32,
    /// Number of retransmissions
    pub retransmissions: u8,
    /// Whether the segment is a zero-window probe
    pub is_zero_window_probe: bool,
}

#[derive(Debug)]
/// A queue for managing TCP retransmissions
pub struct TcpRetransmissionQueue {
    /// The queue of segments to be retransmitted
    queue: VecDeque<RetransmissionSegment>,
    /// Smoothed Round Trip Time (SRTT) for the queue
    srtt: f32,
    /// Measured Round Trip Time (RTT) for the queue
    measured_rtt: f32,
}

impl TcpRetransmissionQueue {
    /// Construct a new `TcpRetransmissionQueue`
    pub fn new() -> Self {
        TcpRetransmissionQueue {
            queue: VecDeque::new(),
            srtt: 1.0,         // Initial SRTT value
            measured_rtt: 0.5, // initial measured rtt
        }
    }

    /// Returns `true` if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get the number of unacknowledged segments in the retransmission queue
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Add a new segment to the queue
    pub fn push(&mut self, packet: TcpPacket, is_zero_window_probe: bool) {
        // Compute the segment's RTO based on the current SRTT
        let rto = Self::compute_rto(self.srtt);

        let segment = RetransmissionSegment {
            packet,
            send_time: Instant::now(),
            rto,
            retransmissions: 0,
            is_zero_window_probe,
        };

        self.queue.push_back(segment);
    }

    /// Update the SRTT using the measured RTT
    fn update_srtt(&mut self, measured_rtt: f32) {
        self.srtt = (ALPHA * self.srtt) + ((1.0 - ALPHA) * measured_rtt);
    }

    /// Compute the RTO using the current SRTT
    pub fn compute_rto(srtt: f32) -> f32 {
        f32::max(RTO_MIN, f32::min(RTO_MAX, BETA * srtt))
    }

    /// Get the duration to sleep before checking for retransmissions
    pub fn sleep_duration(&self) -> Duration {
        let mut min_duration = Duration::from_secs_f32(RTO_MAX);

        for segment in &self.queue {
            let remaining = (segment.rto - segment.send_time.elapsed().as_secs_f32()).max(0.0);
            min_duration = min_duration.min(Duration::from_secs_f32(remaining));
        }

        min_duration
    }

    /// Remove all segments with sequence numbers less than the acknowledgment number from the retransmission queue.
    /// Update the SRTT based on the acknowledgment.
    pub fn acknowledge(&mut self, acknowledgment_number: u32) {
        let ack_time = Instant::now();
        let mut updated_measured_rtt = false;
        let mut i = 0;

        while i < self.queue.len() {
            if let Some(segment) = self.queue.get_mut(i) {
                if segment.packet.header.sequence_number < acknowledgment_number {
                    // If the segment has not been retransmitted, we can use it to update the measured RTT
                    if segment.retransmissions == 0 {
                        self.measured_rtt =
                            ack_time.duration_since(segment.send_time).as_secs_f32();
                        updated_measured_rtt = true;
                    }

                    // Remove the segment from the queue since it has been acknowledged
                    self.queue.remove(i);
                    continue;
                }
            }

            i += 1;
        }

        // Update the SRTT if the measured RTT was updated
        if updated_measured_rtt {
            self.update_srtt(self.measured_rtt);
        }
    }

    /// Retransmit all segments that have timed out (i.e. their RTO has expired)
    pub fn retransmit(&mut self, socket: &TcpNormalSocket, connection: TcpConnection) {
        let mut i = 0;
        while i < self.queue.len() {
            // Get a mutable reference to the current segment
            let segment = self.queue.get_mut(i).unwrap();

            if segment.send_time.elapsed() >= Duration::from_secs_f32(segment.rto) {
                // Retransmit the segment
                socket
                    .tcp_stack
                    .send_tcp_packet(segment.packet.clone(), connection.clone());

                // Update the segment metadata
                segment.retransmissions += 1;
                segment.rto = Self::compute_rto(self.srtt * 2.0); // double this
                segment.send_time = Instant::now();

                // If the maximum number of retransmissions has been exceeded, abort the connection
                if segment.retransmissions > MAX_RETRANSMISSIONS && !segment.is_zero_window_probe {
                    println!(
                        "Max retransmissions ({}) exceeded for sequence number {}. Aborting.",
                        MAX_RETRANSMISSIONS, segment.packet.header.sequence_number
                    );
                    self.queue.clear();
                    socket.set_can_send(false);
                    socket.set_can_read(false);
                    socket.set_can_write(false);
                    socket.set_state(TcpConnectionState::Closed);
                    socket.tcp_stack.remove_socket_table_entry(socket.id);
                    return;
                }

                // Move the segment to the end of the queue
                let segment = self.queue.remove(i).unwrap();
                self.queue.push_back(segment);
                continue;
            }

            i += 1;
        }
    }
}
