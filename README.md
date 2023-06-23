# NetCDF Tile Map Service

A small demo of an on-the-fly NetCDF to TMS tile server using Rust and Leaflet

## Getting Started

### Prerequisites

- Rust installation
- NPM installation

### Running the backend API

```bash
# Clone the repository
git clone github.com/tayden/netcdf-tiles

# Change directory
cd netcdf_tiles

# Start the server
cargo run -p api
```

### Running the web frontend

```bash
# Change directory
cd netcdf-tiles/www

# Install dependencies
npm install

# Start the dev server
npm run dev
```


## Notes

- The api backend is extremely simple and has basically no error handling
- Zooming in too far will not show any tiles. No upsampling or workarounds are implemented
- The API backend may be better in Python, with Rust bindings built using PyO3
  - Alternatively, the whole backend could be written in Python and Rust
- Frontend is Svelte just because it's easy to understand what's happening since it looks like HTML