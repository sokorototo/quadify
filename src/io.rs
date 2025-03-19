use std::{io, task};

use miniquad::fs;
use oneshot::{Receiver, Sender};

struct FileLoadingFuture {
	content: Receiver<Result<Vec<u8>, fs::Error>>,
	waker: Option<Sender<task::Waker>>,
}

impl std::future::Future for FileLoadingFuture {
	type Output = Result<Vec<u8>, fs::Error>;

	fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
		self.waker.take().map(|waker| waker.send(cx.waker().clone()));

		match self.content.try_recv() {
			Ok(res) => task::Poll::Ready(res),
			Err(oneshot::TryRecvError::Empty) => task::Poll::Pending,
			Err(oneshot::TryRecvError::Disconnected) => {
				let error = io::Error::new(io::ErrorKind::Other, "File loading future was dropped");
				task::Poll::Ready(Err(fs::Error::IOError(error)))
			}
		}
	}
}

/// Load a file from the filesystem or http on the web
pub async fn load_file(path: &str) -> Result<Vec<u8>, fs::Error> {
	let (sender, receiver) = oneshot::channel();
	let (waker_sender, waker_receiver) = oneshot::channel::<task::Waker>();

	fs::load_file(path, move |res| {
		let res = res.map(|mut data| {
			data.shrink_to_fit();
			data
		});

		sender.send(res).unwrap();
		waker_receiver.recv().unwrap().wake();
	});

	FileLoadingFuture {
		content: receiver,
		waker: Some(waker_sender),
	}
	.await
}

/// Load a file from the filesystem or http on the web, the parse as a string
pub async fn load_string(path: &str) -> Result<String, fs::Error> {
	let data = load_file(path).await?;
	Ok(String::from_utf8(data).unwrap())
}
