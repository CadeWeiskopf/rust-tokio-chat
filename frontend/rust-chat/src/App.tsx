import React, { useContext } from "react";
import "./App.css";
import { AppContext, AppContextProvider } from "./App.context";
import { Welcome } from "./pages/welcome/Welcome";
import { Lobby } from "./pages/lobby/Lobby";
import { Game } from "./pages/game/Game";

function App() {
  const { localUser, sock, currentMatch } = useContext(AppContext);
  return (
    <div className="app">
      <header>cadew.dev chat</header>
      <main>
        {!sock.websocket && !localUser ? (
          <Welcome />
        ) : currentMatch ? (
          <Game />
        ) : (
          <Lobby />
        )}
      </main>
      <footer>All Rights Reserved, Cade Weiskopf</footer>
    </div>
  );
}

const AppWrapper: React.FC = () => {
  return (
    <AppContextProvider>
      <App />
    </AppContextProvider>
  );
};
export default AppWrapper;
