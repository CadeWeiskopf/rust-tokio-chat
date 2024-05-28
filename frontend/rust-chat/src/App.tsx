import React, { useContext } from "react";
import "./App.css";
import { AppContext, AppContextProvider } from "./App.context";
import { Welcome } from "./pages/welcome/Welcome";
import { Lobby } from "./pages/lobby/Lobby";

function App() {
  const { localUser, sock } = useContext(AppContext);
  return (
    <div className="app">
      <header>cadew.dev chat</header>
      <main>
        {/* <Game /> */}
        {!sock.websocket && !localUser ? <Welcome /> : <Lobby />}
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
