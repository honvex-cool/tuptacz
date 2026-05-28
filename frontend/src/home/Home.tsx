import "./Home.css"

export default function Home() {
    return (<main id="home">

        <img src="tuptacz-light.png" id="main-img" />
        <p className="title text-primary">
            Tuptacz
        </p>

        <p className="caveat-400 poem">
            Gdzie jest słońce, kiedy śpi?
            <br />
            Czy wilk zawsze bywa zły?
            <br />
            Dokąd tupta nocą jeż?
            <br />
            Możesz wiedzieć, jeśli chcesz.
        </p>
        <div id="home-btns">
            <a href="/tuptach" className="btn-primary">TuptaCH</a>
            <a href="/jakdotuptam" className="btn-primary">JakDotuptam</a>
        </div>

    </main>)
}