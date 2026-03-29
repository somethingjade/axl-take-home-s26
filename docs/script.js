const API_BASE = "https://axl-take-home-s26-backend.onrender.com";
let session_id;
window.onload = async () => {
	 const res = await fetch(`${API_BASE}/new-session`, {
			method: "POST",
			headers: {"Content-Type": "application/json"},
	 });
	 ({ session_id } = await res.json());
	await send("");
}

const send = async (msg) => {
	const chat = document.getElementById("chat");
	chat.innerText += "\n\nPlease wait...";
	scrollToBottom();
	const res = await fetch(`${API_BASE}/play`, {
		method: "POST",
		headers: {"Content-Type": "application/json"},
		body: JSON.stringify({ session_id, message: msg })
	});
	const data = await res.json();
	chat.innerText = chat.innerText.replace(
		"\n\nPlease wait...",
		data.reply
	);
	scrollToBottom();
}

const sendBtn = async () => {
	const input = document.getElementById("input");
	console.log(session_id);
	const msg = input.value;
	input.value = "";
	try {
		await send(msg)
	} catch {
		chat.innerText += "\n[Server waking up or error]";
	}
}

function scrollToBottom() {
	const chat = document.getElementById("chat");
	if (!chat) return;
	chat.scrollTop = chat.scrollHeight;
}

window.addEventListener("load", () => {
	const chat = document.getElementById("chat");
	const input = document.getElementById("input");
	if (input) input.focus();
	document.addEventListener("keydown", (e) => {
		if (e.key === "Enter" && document.activeElement === input) {
			e.preventDefault();
			sendBtn();
		}
	});
	if (chat) {
		let lastText = chat.innerText;

		const observer = new MutationObserver(() => {
			if (chat.innerText === lastText) return;
			lastText = chat.innerText;

			chat.style.opacity = "0";
			chat.style.transform = "translateY(6px)";

			requestAnimationFrame(() => {
				chat.style.transition = "opacity 0.2s ease, transform 0.2s ease";
				chat.style.opacity = "1";
				chat.style.transform = "translateY(0)";
			});
		});
		observer.observe(chat, {
			childList: true,
			subtree: true,
			characterData: true
		});
	}
});
