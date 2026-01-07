import { createBrowserRouter } from "react-router-dom";
import Layout from "@/layout/Layout"; // 你的布局组件
import TableTest from "@/pages/TableTest.tsx";
import App from "../App.tsx";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      {
        index: true,
        element: <TableTest />,
      },
    ],
  },
]);
