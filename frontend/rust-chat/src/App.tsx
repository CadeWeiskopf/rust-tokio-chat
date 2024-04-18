import React from "react";
import "./App.css";
import { Chat } from "./components/chat/Chat";

function App() {
  return (
    <div className="app">
      <header>cadew.dev chat</header>
      <main>
        <Chat />
      </main>
      <footer>All Rights Reserved, Cade Weiskopf</footer>
    </div>
  );
}

export default App;
