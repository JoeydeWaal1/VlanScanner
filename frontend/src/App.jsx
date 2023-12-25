import { useEffect, useState } from "react";
import axios from "axios"


function App() {
  const [devices, setDevices] = useState([]);
  const [selected, setSelected] = useState(null);
  const [data, setData] = useState([]);

  useEffect(() => {
    console.log("fetching devices");
    axios.get("http://localhost:3001/devices")
    .then((r) => setDevices(r.data))
    .catch((e) => console.log(e));
  }, [])

  useEffect(() => {
    if (selected != null){
      console.log(selected.id);
        let url = new URL(`/ws/${selected.id}`, window.location.href)
        url.port = "3001";
        url.protocol = url.protocol.replace("http","ws");
        console.log(url.href);

        let ws = new WebSocket(url.href);
        ws.onmessage = (m) => {
          const msg = JSON.parse(m.data);
          setData((p) => [...p, msg]);
          console.log(msg);
        }
    }
  }, [selected]);
  console.log(data)

  return (
    <>
    {
      selected == null ?      
      <div className="flex flex-col">
      <h1 className="m-auto font-bold">Selecteer een interface</h1>

        {devices.map((d) =>
          <button
            key={d.name}
            className="hover:bg-gray-400"
            onClick={() => setSelected(d)}>{d.name}
          </button>)}
        </div> :
      <h1>Selected: {selected.name}</h1>
    }
      {
        data.map((d,i)=> <div key={i}>{d.src} {d.dst}</div>)
      }
    </>
 
  );
}

export default App;
