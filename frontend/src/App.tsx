import { Routes, Route } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import Providers from './pages/Providers'
import McpServers from './pages/McpServers'
import CliTools from './pages/CliTools'
import ProxySettings from './pages/ProxySettings'
import Settings from './pages/Settings'

function App() {
  return (
    <>
      <Toaster position="top-right" />
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="providers" element={<Providers />} />
          <Route path="mcp" element={<McpServers />} />
          <Route path="cli-tools" element={<CliTools />} />
          <Route path="proxy" element={<ProxySettings />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </>
  )
}

export default App
