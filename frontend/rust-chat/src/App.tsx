import React, { useContext } from "react";
import "./App.css";
import { Chat } from "./components/chat/Chat";
import { AppContext, AppContextProvider } from "./App.context";
import { Welcome } from "./components/welcome/Welcome";

function App() {
  const { localUser } = useContext(AppContext);
  return (
    <div className="app">
      <header>cadew.dev chat</header>
      <main>{localUser === null ? <Welcome /> : <Chat />}</main>
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
