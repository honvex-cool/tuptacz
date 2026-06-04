import './index.css'
import App from './App.tsx'

import ReactDOM from "react-dom/client";
import { createBrowserRouter } from "react-router";
import { RouterProvider } from "react-router/dom";
import Main from './Main.tsx';
import JakDotuptam from './jakdotuptam/JakDotuptam.tsx';
import Home from './home/Home.tsx';

const router = createBrowserRouter([
  {
    path: "/",
    element: <Home/>
  },
  {
    path: "/tuptach",
    element: <Main title="TuptaCH"> <App /> </Main>
  },
  {
    path: "/jakdotuptam",
    element: (<Main title="JakDotuptam">
      <JakDotuptam />
    </Main>)
  }
]);

const root = document.getElementById("root")!;

ReactDOM.createRoot(root).render(
  <RouterProvider router={router} />,
);