<script>
  import L from 'leaflet';

  let map;
  const initialView = [49, -123];
  const initialZoom = 7;

  // BEGIN SECTION OF INTEREST
  L.TileLayer.ChlConc = L.TileLayer.extend({
    getTileUrl: function (coords) {
      var max_y_for_z = Math.pow(2, coords.z) - 1;
      coords.y = max_y_for_z - coords.y;

      // TODO: Date should be dynamic
      return `http://127.0.0.1:8000/chl_conc/2023/06/08/${coords.x}/${coords.y}/${coords.z}`;
    },
    getAttribution: function () {
      return "<a href='https://hakai.org'>Hakai + SpectralLab</a>"
    },
  });

  L.tileLayer.chl_conc = function () {
    return new L.TileLayer.ChlConc();
  }
  // END SECTION OF INTEREST

  function createMap(container) {
    let m = L.map(container, {preferCanvas: true, maxZoom: 9 }).setView(initialView, initialZoom);
    L.tileLayer(
      'https://{s}.basemaps.cartocdn.com/rastertiles/voyager/{z}/{x}/{y}{r}.png',
      {
        attribution: `&copy;<a href="https://www.openstreetmap.org/copyright" target="_blank">OpenStreetMap</a>,
	        &copy;<a href="https://carto.com/attributions" target="_blank">CARTO</a>`,
        subdomains: 'abcd',
      },
    ).addTo(m);

    // Add the custom tile layer here
    L.tileLayer.chl_conc({tms: true, maxZoom: 9}).addTo(m);

    return m;
  }

  function resizeMap() {
    if (map) {
      map.invalidateSize();
    }
  }

  function mapAction(container) {
    map = createMap(container);

    return {
      destroy: () => {
        map.remove();
        map = null;
      },
    };
  }
</script>

<svelte:window on:resize={resizeMap}/>

<link crossorigin="" href="https://unpkg.com/leaflet@1.6.0/dist/leaflet.css"
      integrity="sha512-xwE/Az9zrjBIphAcBb3F6JVqxf46+CDLwfLMHloNu6KEQCAWi6HcDUbeOfBIptF7tcCzusKFjFw2yuvEpDL9wQ=="
      rel="stylesheet"/>

<div class="map" use:mapAction></div>

<style>
    .map {
        height: 100%;
        width: 100%;
    }
</style>