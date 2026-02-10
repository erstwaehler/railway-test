# railway-test

```
 ____       _ _                     _____         _   
|  _ \ __ _(_) |_      ____ _ _   _|_   _|__  ___| |_ 
| |_) / _` | | \ \ /\ / / _` | | | | | |/ _ \/ __| __|
|  _ < (_| | | |\ V  V / (_| | |_| | | |  __/\__ \ |_ 
|_| \_\__,_|_|_| \_/\_/ \__,_|\__, | |_|\___||___/\__|
                              |___/                    
```

## Monorepo Structure

This is a turborepo-powered monorepo using bun workspaces.

### Setup

```bash
bun install
```

### Development

```bash
bun run dev      # Run all apps in development mode
bun run build    # Build all apps
bun run lint     # Lint all apps
```

### Workspace Structure

```
apps/
  ├── *          # Application workspaces
```

### Tech Stack

- **Package Manager**: Bun
- **Build System**: Turborepo
- **Workspaces**: Bun Workspaces