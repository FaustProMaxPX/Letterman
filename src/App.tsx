import { useState } from "react";
import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import "./App.css";
import { Container, Typography } from "@mui/material";
import { Router } from "react-router-dom";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="App">
      <Router>
        <Container>
          <Typography variant="h1">Letter Man</Typography>
        </Container>
      </Router>
    </div>
  );
}

export default App;
