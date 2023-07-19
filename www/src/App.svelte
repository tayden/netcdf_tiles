<script>
    import L from 'leaflet';

    let map;
    const initialView = [49, -123];
    const initialZoom = 7;

    // BEGIN SECTION OF INTEREST
    L.TileLayer.ChlConc = L.TileLayer.extend({
        getTileUrl: function (coords) {
            // TODO: Date should be dynamic
            return `http://127.0.0.1:8000/chl_conc_mean/2023/07/01/${coords.x}/${coords.y}/${coords.z}?max_value=40&min_value=0.15&log_scale&gradient=viridis`;
        },
        getAttribution: function () {
            return "<a href='https://hakai.org' target='_blank'>Hakai Institute</a>, <a href='https://http://uvicspectral.com/' target='_blank'>Spectral Laboratory</a>"
        },
    });

    L.tileLayer.chl_conc = function () {
        return new L.TileLayer.ChlConc();
    }

    // END SECTION OF INTEREST


    L.GridLayer.DebugCoords = L.GridLayer.extend({
        createTile: function (coords) {
            var tile = document.createElement('div');
            tile.innerHTML = [coords.x, coords.y, coords.z].join(', ');
            tile.style.outline = '1px solid red';
            tile.style.color = 'red';
            return tile;
        }
    });

    L.gridLayer.debugCoords = function(opts) {
        return new L.GridLayer.DebugCoords(opts);
    };

    function createMap(container) {
        let m = L.map(container, {
            preferCanvas: true,
            maxBounds: [
                [44.887012, -111.137695], // southwest corner
                [59.92199, -144.624023] // northeast corner
            ]
        })
            .setView(initialView, initialZoom);
        L.tileLayer(
            'https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png',
            {
                attribution: `&copy;<a href="https://www.openstreetmap.org/copyright" target="_blank">OpenStreetMap</a>,
	        &copy;<a href="https://carto.com/attributions" target="_blank">CARTO</a>`,
                subdomains: 'abcd',
            },
        ).addTo(m);

        // Add the custom tile layer here
        L.tileLayer.chl_conc({maxZoom: 9}).addTo(m);
        // L.gridLayer.debugCoords().addTo(m);
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