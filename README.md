# axl-take-home-s26

## Getting started
### 1. Clone the repository
```
git clone https://github.com/somethingjade/axl-take-home-s26.git
cd axl-take-home-s26
```

---

## Server (Backend)
### Prerequisites:
- [Rust + Cargo](https://rust-lang.org/)
### Setup
1. `cd server`
2. Create a `.env` file and add your [Groq](https://groq.com/) API key
```
GROQ_API_KEY=your_api_key_here
```
3. Run the server:
```
cargo run --release
```
The backend will start at:
http://localhost:3000

---

## App (Frontend)
### Setup
Open `docs/script.js` and update the API base URL:
```
const API_BASE = "http://localhost:3000";
```

---

### Run the frontend
Open the following file in your browser:
```
docs/index.html
```
---
## Notes
- The backend must be running before using the frontend
- The frontend communiates with the backend via HTTP requests
- Each session is managed using a unique session ID
