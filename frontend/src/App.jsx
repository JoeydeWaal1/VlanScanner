import { useEffect, useState } from "react";
import axios from "axios"
import {XYPlot as Plot, XAxis, YAxis, HorizontalGridLines, DiscreteColorLegend, VerticalGridLines, VerticalBarSeries, makeVisFlexible} from 'react-vis';
import '../node_modules/react-vis/dist/style.css';


function App() {
  const [devices, setDevices] = useState([]);
  const [selected, setSelected] = useState(null);
  const [data, setData] = useState([]);

  const XYPlot = makeVisFlexible(Plot);

  useEffect(() => {
    console.log("fetching devices");
    axios.get("/devices")
    .then((r) => setDevices(r.data))
    .catch((e) => console.log(e));
  }, [])

  useEffect(() => {
    if (selected != null){
      console.log(selected.id);
        let url = new URL(`/ws/${selected.id}`, window.location.href)
        // url.port = "3003";
        url.protocol = url.protocol.replace("http","ws");
        console.log(url.href);

        let ws = new WebSocket(url.href);
        ws.onmessage = (m) => {
          const msg = JSON.parse(m.data);
          setData((p) => [...p, msg]);
          // console.log(msg);
        }
    }
  }, [selected]);
  // console.log(data)
  const vlans = data.filter((d) => d.vlan != null).map((d) => d.vlan).sort((a,b)=> a-b);
  const unique_vlans = [...new Set(vlans)];

  let graph_data = unique_vlans.map( (v)=> ({
    x: `Vlan ${v}`,
    y : vlans.filter((v2) => v==v2).length
  }))

  graph_data = [
    {
      x: "Untagged",
      y: data.length - vlans.length
    },
    ...graph_data
  ]
  console.log(graph_data)

  return (
    <>
    {
      data.length != 0 &&
      <>
    <XYPlot xType="ordinal" height={700} xDistance={100}>
        <VerticalGridLines />
        <HorizontalGridLines />
        <XAxis title='Vlans'/>
        <YAxis />
        <VerticalBarSeries data={graph_data} />
    </XYPlot>
    {/* <p>Aantal vlans: {vlans.length}</p> */}
      <table>
        <tbody className={`grid grid-cols-${Math.ceil(graph_data.length / 10)}`}>
      {
        graph_data.map((data, i) =>
          <tr key={i} className="w-full px-1">
            <td className="font-bold w-full text-left">
            {data.x}
            </td>
            <td>
              {data.y}
            </td>
          </tr>
        )
      }
      </tbody>
      </table>
      </>
    }
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
        data.map((d,i)=> <div key={i}>{d.src} {d.dst} {d.vlan || "geen vlan"}</div>)
      }
    </>
 
  );
}

export default App;
