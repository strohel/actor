use crate::actors::*;
use anyhow::Error;
use env_logger::Env;
use tonari_actor::{Addr, System, SystemCallbacks};

/// A simplistic representation of a MediaFrame, they just hold frame counters.
pub enum MediaFrame {
    Video(usize),
    Audio(usize),
}

/// A simplistic representation of an encoded MediaFrame, they just hold frame counters.
pub enum EncodedMediaFrame {
    Video(usize),
    Audio(usize),
}

mod actors {
    use crate::{EncodedMediaFrame, MediaFrame};
    use anyhow::{bail, Error};
    use log::info;
    use std::{thread, time::Duration};
    use tonari_actor::{Actor, Context, Recipient};

    // Messages
    pub enum VideoCaptureMessage {
        Capture,
    }

    pub enum AudioCaptureMessage {
        Capture,
    }

    // Plumbing
    pub struct ShutdownActor;

    impl Actor for ShutdownActor {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = ();

        fn name() -> &'static str {
            "ShutdownActor"
        }

        fn handle(
            &mut self,
            context: &mut Self::Context,
            _msg: Self::Message,
        ) -> Result<(), Self::Error> {
            context.system_handle.shutdown().expect("ShutdownActor failed to shutdown system");
            Ok(())
        }
    }

    // Egress pipeline
    pub struct VideoCapturer {
        frame_counter: usize,
        next: Recipient<MediaFrame>,
    }

    impl VideoCapturer {
        pub fn new(next: Recipient<MediaFrame>) -> Self {
            Self { frame_counter: 0, next }
        }
    }

    impl Actor for VideoCapturer {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = VideoCaptureMessage;

        fn name() -> &'static str {
            "VideoCaptureActor"
        }

        fn handle(
            &mut self,
            context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            match message {
                VideoCaptureMessage::Capture => {
                    // Simulate a video frame capture
                    std::thread::sleep(Duration::from_millis(16));

                    self.next.send(MediaFrame::Video(self.frame_counter))?;
                    self.frame_counter += 1;

                    context.myself.send(VideoCaptureMessage::Capture)?;
                },
            }
            Ok(())
        }
    }

    pub struct VideoEncoder {
        next: Recipient<EncodedMediaFrame>,
    }

    impl VideoEncoder {
        pub fn new(next: Recipient<EncodedMediaFrame>, message: &str) -> Self {
            info!("VideoEncoder starting: {message}.");
            Self { next }
        }
    }

    impl Actor for VideoEncoder {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = MediaFrame;

        fn name() -> &'static str {
            "VideoEncodeActor"
        }

        fn handle(
            &mut self,
            _context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            match message {
                MediaFrame::Video(frame_counter) => {
                    // Simulate some encoding work
                    thread::sleep(Duration::from_millis(7));
                    self.next.send(EncodedMediaFrame::Video(frame_counter))?;
                },
                MediaFrame::Audio(_) => {
                    bail!("Why did you give the VideoEncodeActor an audio MediaFrame?");
                },
            }

            Ok(())
        }
    }

    pub struct AudioCapturer {
        frame_counter: usize,
        next: Recipient<MediaFrame>,
    }

    impl AudioCapturer {
        pub fn new(next: Recipient<MediaFrame>) -> Self {
            Self { frame_counter: 0, next }
        }
    }

    impl Actor for AudioCapturer {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = AudioCaptureMessage;

        fn name() -> &'static str {
            "AudioCaptureActor"
        }

        fn handle(
            &mut self,
            context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            match message {
                AudioCaptureMessage::Capture => {
                    // Simulate an audio frame capture
                    std::thread::sleep(Duration::from_millis(10));

                    self.next.send(MediaFrame::Audio(self.frame_counter))?;
                    self.frame_counter += 1;

                    context.myself.send(AudioCaptureMessage::Capture)?;
                },
            }

            Ok(())
        }
    }

    pub struct AudioEncoder {
        next: Recipient<EncodedMediaFrame>,
    }

    impl AudioEncoder {
        pub fn new(next: Recipient<EncodedMediaFrame>, message: &str) -> Self {
            info!("AudioEncoder starting: {message}.");
            Self { next }
        }
    }

    impl Actor for AudioEncoder {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = MediaFrame;

        fn name() -> &'static str {
            "AudioEncodeActor"
        }

        fn handle(
            &mut self,
            _context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            match message {
                MediaFrame::Audio(frame_counter) => {
                    // Simulate some encoding work
                    thread::sleep(Duration::from_millis(3));
                    self.next.send(EncodedMediaFrame::Audio(frame_counter))?;
                },
                MediaFrame::Video(_) => {
                    bail!("Why did you give the AudioEncodeActor a video MediaFrame?");
                },
            }

            Ok(())
        }
    }

    pub struct NetworkSender {
        next: Recipient<EncodedMediaFrame>,
    }

    impl NetworkSender {
        pub fn new(next: Recipient<EncodedMediaFrame>) -> Self {
            Self { next }
        }
    }

    impl Actor for NetworkSender {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = EncodedMediaFrame;

        fn name() -> &'static str {
            "NetworkSenderActor"
        }

        fn handle(
            &mut self,
            _context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            // Add some fake packetization and network latency here
            thread::sleep(Duration::from_millis(30));
            self.next.send(message)?;

            Ok(())
        }
    }

    // Ingress pipeline
    pub struct NetworkReceiver {
        audio_next: Recipient<EncodedMediaFrame>,
        video_next: Recipient<EncodedMediaFrame>,
    }

    impl NetworkReceiver {
        pub fn new(
            audio_next: Recipient<EncodedMediaFrame>,
            video_next: Recipient<EncodedMediaFrame>,
        ) -> Self {
            Self { audio_next, video_next }
        }
    }

    impl Actor for NetworkReceiver {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = EncodedMediaFrame;

        fn name() -> &'static str {
            "NetworkReceiverActor"
        }

        fn handle(
            &mut self,
            _context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            match message {
                EncodedMediaFrame::Video(_) => {
                    self.video_next.send(message)?;
                },
                EncodedMediaFrame::Audio(_) => {
                    self.audio_next.send(message)?;
                },
            }

            Ok(())
        }
    }

    pub struct VideoDecoder {
        next: Recipient<MediaFrame>,
    }

    impl VideoDecoder {
        pub fn new(next: Recipient<MediaFrame>) -> Self {
            Self { next }
        }
    }

    impl Actor for VideoDecoder {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = EncodedMediaFrame;

        fn name() -> &'static str {
            "VideoDecodeActor"
        }

        fn handle(
            &mut self,
            _context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            match message {
                EncodedMediaFrame::Video(frame_counter) => {
                    self.next.send(MediaFrame::Video(frame_counter))?;
                },
                EncodedMediaFrame::Audio(_) => {
                    bail!("Why did you give the VideoDecodeActor an audio EncodedMediaFrame?");
                },
            }

            Ok(())
        }
    }

    pub struct AudioDecoder {
        next: Recipient<MediaFrame>,
    }

    impl AudioDecoder {
        pub fn new(next: Recipient<MediaFrame>) -> Self {
            Self { next }
        }
    }

    impl Actor for AudioDecoder {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = EncodedMediaFrame;

        fn name() -> &'static str {
            "AudioDecodeActor"
        }

        fn handle(
            &mut self,
            _context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            match message {
                EncodedMediaFrame::Audio(frame_counter) => {
                    self.next.send(MediaFrame::Audio(frame_counter))?;
                },
                EncodedMediaFrame::Video(_) => {
                    bail!("Why did you give the AudioDecodeActor a video EncodedMediaFrame?");
                },
            }

            Ok(())
        }
    }

    pub struct AudioPlayback {}

    impl AudioPlayback {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Actor for AudioPlayback {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = MediaFrame;

        fn name() -> &'static str {
            "AudioPlaybackActor"
        }

        fn handle(
            &mut self,
            _context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            match message {
                MediaFrame::Audio(frame_counter) => {
                    println!("🔊 Playing back audio frame {}", frame_counter);
                },
                MediaFrame::Video(_) => {
                    bail!("Why did you give the AudioPlaybackActor a video MediaFrame?");
                },
            }

            Ok(())
        }
    }

    pub struct VideoDisplay {}

    impl VideoDisplay {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Actor for VideoDisplay {
        type Context = Context<Self::Message>;
        type Error = Error;
        type Message = MediaFrame;

        fn name() -> &'static str {
            "VideoDisplayActor"
        }

        fn handle(
            &mut self,
            context: &mut Self::Context,
            message: Self::Message,
        ) -> Result<(), Self::Error> {
            match message {
                MediaFrame::Video(frame_counter) => {
                    println!("🖼 Display video frame {}", frame_counter);

                    if frame_counter >= 360 {
                        println!(
                            "We've received 360 video frames, shutting down the actor system!"
                        );
                        let _ = context.system_handle.shutdown();
                    }
                },
                MediaFrame::Audio(_) => {
                    bail!("Why did you give the VideoDisplayActor an audio MediaFrame?");
                },
            }

            Ok(())
        }
    }
}

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let system_callbacks = SystemCallbacks {
        preshutdown: Some(Box::new(move || {
            println!("The actor system is stopping, this is the preshutdown hook");
            Ok(())
        })),
        ..SystemCallbacks::default()
    };

    let mut system = System::with_callbacks("main", system_callbacks);

    // TODO - Add some extra "config" actors to adjust things like video capture exposure,
    //        or playback volume.

    // Handle Ctrl-C
    let shutdown_addr = system.spawn(ShutdownActor {})?;
    ctrlc::set_handler(move || {
        shutdown_addr.send(()).expect("failed to send shutdown message");
    })?;

    // Wire up the actors
    let display_addr = Addr::default();

    // Receiving side
    let audio_playback_actor = system.spawn(AudioPlayback::new())?;

    let video_decode_addr = system.spawn(VideoDecoder::new(display_addr.recipient()))?;
    let audio_decode_addr = system.spawn(AudioDecoder::new(audio_playback_actor.recipient()))?;

    let network_receiver_addr = system.spawn(NetworkReceiver::new(
        audio_decode_addr.recipient(),
        video_decode_addr.recipient(),
    ))?;

    // Sending side
    let network_sender_addr =
        system.spawn(NetworkSender::new(network_receiver_addr.recipient()))?;

    // I really want to initialize audio first.
    let audio_encode_addr =
        system.spawn(AudioEncoder::new(network_sender_addr.recipient(), "hello"))?;
    let video_encode_addr =
        system.spawn(VideoEncoder::new(network_sender_addr.recipient(), "hi"))?;

    // Dtto. :]
    let audio_capture_addr = system.spawn(AudioCapturer::new(audio_encode_addr.recipient()))?;
    let video_capture_addr = system.spawn(VideoCapturer::new(video_encode_addr.recipient()))?;

    // Kick off the pipeline
    audio_capture_addr.send(AudioCaptureMessage::Capture)?;
    video_capture_addr.send(VideoCaptureMessage::Capture)?;

    // The display actor may spawn an OS window which in some cases must run
    // on the main application thread.
    let display_actor = VideoDisplay::new();
    system.prepare(display_actor).with_addr(display_addr).run_and_block()?;

    Ok(())
}
