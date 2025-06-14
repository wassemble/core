# Molecular Architecture for Workflow Automation Tool

## Core Molecules (Domain Concepts)

### 1. `workflow-engine/` - The Heart of Execution
**Domain:** Workflow execution and orchestration
```
workflow-engine/
├── workflow-engine.ts     // Core execution logic
├── wasm-runner.ts         // Wasmtime integration
├── execution-context.ts   // Runtime state management
├── schema.sql            // Workflow definitions, runs, states
└── main.ts              // Standalone runner: node run workflow-engine input.json
```

**Responsibilities:**
- Load and validate workflow definitions
- Execute WASM components via wasmtime
- Manage execution state and context
- Handle data flow between nodes
- **Runnable alone:** Can execute workflows headlessly for CI/CD, testing, or server-side automation

### 2. `component-builder/` - WASM Component Management
**Domain:** Building and managing workflow components
```
component-builder/
├── component-builder.ts   // Cargo component integration
├── wasm-registry.ts      // Component storage and versioning
├── component-validator.ts // Validate WASM interfaces
├── schema.sql           // Component metadata, versions
└── main.ts             // Standalone: build and test components
```

**Responsibilities:**
- Interface with `cargo component` to build WASM modules
- Store and version workflow components
- Validate component interfaces and compatibility
- **Runnable alone:** CLI tool for component development and testing

### 3. `workflow-designer/` - Visual Flow Editor
**Domain:** Workflow creation and editing interface
```
workflow-designer/
├── workflow-designer.ts  // Core designer logic
├── svelteflow-adapter.ts // SvelteFlow integration
├── node-palette.ts      // Available components UI
├── canvas-state.ts      // Designer state management
└── main.ts             // Standalone designer app
```

**Responsibilities:**
- Visual workflow creation using SvelteFlow
- Node palette management
- Canvas state and validation
- Export workflow definitions
- **Runnable alone:** Pure designer interface for workflow creation

### 4. `execution-monitor/` - Runtime Observability
**Domain:** Workflow execution monitoring and debugging
```
execution-monitor/
├── execution-monitor.ts  // Real-time execution tracking
├── trace-collector.ts   // Execution trace management
├── output-manager.ts    // Handle workflow outputs
├── debug-interface.ts   // Debugging tools
├── schema.sql          // Traces, outputs, metrics
└── main.ts            // Standalone monitoring dashboard
```

**Responsibilities:**
- Collect and display execution traces
- Manage workflow outputs and artifacts
- Provide debugging interface
- Real-time execution monitoring
- **Runnable alone:** Monitoring dashboard for production workflows

### 5. `node-configurator/` - Node Settings Management
**Domain:** Individual node configuration and validation
```
node-configurator/
├── node-configurator.ts  // Node configuration logic
├── settings-validator.ts // Validate node settings
├── schema-generator.ts  // Generate settings schemas from WASM
├── ui-generator.ts      // Dynamic settings UI
└── main.ts             // Standalone node config tool
```

**Responsibilities:**
- Dynamic settings UI generation based on component schemas
- Settings validation and serialization
- Schema introspection from WASM components
- **Runnable alone:** Component configuration and testing tool

### 6. `ai-assistant/` - Intelligent Workflow Help
**Domain:** AI-powered workflow assistance
```
ai-assistant/
├── ai-assistant.ts      // Core AI integration
├── workflow-analyzer.ts // Analyze and suggest improvements
├── component-suggester.ts // Suggest relevant components
├── prompt-manager.ts    // Manage AI prompts and context
└── main.ts             // Standalone AI chat interface
```

**Responsibilities:**
- Analyze workflows and suggest improvements
- Help users find relevant components
- Generate workflow templates
- Provide contextual help
- **Runnable alone:** AI assistant for workflow development

### 7. `collaboration-hub/` - Real-time Collaboration
**Domain:** Multi-user workflow collaboration
```
collaboration-hub/
├── collaboration-hub.ts // Core collaboration logic
├── websocket-server.ts  // Real-time communication
├── conflict-resolver.ts // Handle concurrent edits
├── presence-manager.ts  // User presence tracking
├── schema.sql          // Sessions, presence, changes
└── main.ts            // Standalone collaboration server
```

**Responsibilities:**
- WebSocket-based real-time collaboration
- Conflict resolution for concurrent edits
- User presence and cursor tracking
- Change synchronization
- **Runnable alone:** Collaboration server for team workflows

## Shared Libraries (`lib/`)

```
lib/
├── wasm-interface.ts    // Common WASM component interface
├── workflow-schema.ts   // Workflow definition schemas
├── websocket-client.ts  // Shared WebSocket utilities
├── database-service.ts  // Configurable DB service (in-memory/remote)
├── event-bus.ts        // Cross-module event system
└── auth-service.ts     // Authentication utilities
```

## Aggregator Modules

### `workflow-studio/` - Full Development Environment
```
workflow-studio/
└── main.ts  // Combines designer + configurator + ai-assistant + collaboration
```

### `workflow-server/` - Production Runtime
```
workflow-server/
└── main.ts  // Combines engine + monitor + collaboration for production
```

### `workflow-cli/` - Command Line Interface
```
workflow-cli/
└── main.ts  // Combines engine + builder for headless operations
```

## Dependency Graph

```
workflow-studio → workflow-designer
                → node-configurator  
                → ai-assistant
                → collaboration-hub

workflow-server → workflow-engine
                → execution-monitor
                → collaboration-hub

workflow-cli → workflow-engine
             → component-builder

All modules → lib/* (shared utilities)
```

## Key Benefits of This Structure

### 1. **Independent Development & Testing**
- Test component building: `node run component-builder some-component.rs`
- Test workflow execution: `node run workflow-engine my-workflow.json`
- Test designer UI: `node run workflow-designer` (opens browser)
- Test monitoring: `node run execution-monitor` (dashboard)

### 2. **Flexible Deployment**
- **Development:** Run `workflow-studio` with all features
- **Production:** Run `workflow-server` for execution + monitoring
- **CI/CD:** Use `workflow-cli` for automated testing
- **Microservice Migration:** Each molecule can become a separate service

### 3. **Clean Feature Removal**
Don't need AI assistance? Delete `ai-assistant/` folder and remove from aggregators.
Don't need collaboration? Delete `collaboration-hub/` folder.

### 4. **Domain-Focused Testing**
```javascript
// Test workflow execution without UI complexity
test('workflow executes correctly', async () => {
  const db = DatabaseService('in-memory');
  const engine = new WorkflowEngine(db);
  
  const result = await engine.execute(simpleWorkflow);
  expect(result.status).toBe('completed');
});
```

### 5. **Technology Flexibility**
- Swap SvelteFlow for React Flow? Only affects `workflow-designer/`
- Change from wasmtime to another WASM runtime? Only affects `workflow-engine/`
- Different AI provider? Only affects `ai-assistant/`

## Integration Points

### Event-Driven Communication
```javascript
// workflow-designer publishes workflow changes
EventBus.publish('workflow.changed', workflowDefinition);

// collaboration-hub listens and broadcasts to other users
EventBus.subscribe('workflow.changed', (workflow) => {
  WebSocketServer.broadcast('workflow.update', workflow);
});
```

### Shared Schemas
```javascript
// lib/workflow-schema.ts defines the contract
interface WorkflowDefinition {
  nodes: WorkflowNode[];
  connections: Connection[];
  metadata: WorkflowMetadata;
}

// All modules use the same schema
```

This architecture gives you the "microservices feel" with monolith simplicity. Each domain concept is fully self-contained, testable, and runnable independently, while still being able to compose into a complete workflow automation platform.
