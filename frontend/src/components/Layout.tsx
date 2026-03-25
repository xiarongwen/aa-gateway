import { Link, NavLink } from 'react-router-dom'
import { Server, Terminal } from 'lucide-react'
import { Outlet } from 'react-router-dom'

const navigation = [
  // { name: 'Dashboard', href: '/', icon: LayoutDashboard },
  { name: 'Providers', href: '/providers', icon: Server },
  // { name: 'MCP Servers', href: '/mcp', icon: Puzzle },
  { name: 'CLI Tools', href: '/cli-tools', icon: Terminal },
  // { name: 'Proxy', href: '/proxy', icon: Network },
  // { name: 'Settings', href: '/settings', icon: Settings },
]

export default function Layout() {
  return (
    <div className="min-h-screen bg-gray-50">
      <div className="hidden lg:fixed lg:inset-y-0 lg:flex lg:w-64 lg:flex-col">
        <div className="flex flex-col flex-1 bg-white border-r">
          <div className="flex items-center h-16 px-4 border-b">
            <Link to="/" className="text-xl font-bold text-gray-900">
              AI Gateway
            </Link>
          </div>
          <nav className="flex-1 p-4 space-y-1">
            {navigation.map((item) => (
              <NavLink
                key={item.name}
                to={item.href}
                className={({ isActive }) =>
                  `flex items-center px-4 py-2 text-sm font-medium rounded-md ${
                    isActive
                      ? 'bg-indigo-50 text-indigo-600'
                      : 'text-gray-700 hover:bg-gray-50'
                  }`
                }
              >
                <item.icon className="w-5 h-5 mr-3" />
                {item.name}
              </NavLink>
            ))}
          </nav>
        </div>
      </div>

      <div className="lg:pl-64">
        <main className="p-4 lg:p-8">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
