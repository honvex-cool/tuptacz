import type { ReactNode } from "react";
import { Link } from "react-router-dom";

import "./Main.css"

type MainProps = {
    children?: ReactNode;
    title: string;
};


function link(to: string, text: string) {
    const path = window.location.pathname;
    const className = path === to ? "active" : "";
    return (
        <Link to={to} className={className}>
            {text}
        </Link>
    );
}

function TopNav(props: { title: string }) {
    return <div className="top-nav">
        <div className="top-nav-left">
            <div className="nav-title">
                <img src="tuptacz-light.png" className="nav-logo" />
                <span className="text-primary">Tuptacz</span>
            </div>
        </div>
        <div className="nav-links">
            {link("/tuptach", "TuptaCH")}
            {link("/jakdotuptam", "JakDotuptam")}
        </div>
    </div>
}

export default function Main(props: MainProps) {
    return (
        <div className="main">
            <title>{props.title}</title>
            <TopNav title={props.title} />
            {props.children}
        </div>
    );
}