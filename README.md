# Mesh Framework

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Python](https://img.shields.io/badge/python-3.8+-blue.svg)](https://python.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/meshframework/mesh)

> **Filter your AI agent's memory and make room for what matters.**

Mesh introduces a sophisticated approach to AI memory management, enhancing agent performance by prioritizing what truly matters. By dynamically removing obsolete memories and enabling seamless memory sharing between agents, Mesh empowers AI systems to maintain peak operational speed and precision while setting a new standard in adaptive intelligence.

## 🚀 Features

- **Memory Filtering**: Automatically discard stale and irrelevant data to keep agent memory optimized
- **Memory Sharing**: Enable seamless memory synchronization between multiple AI agents
- **Dynamic Memory Management**: Systematically clear outdated information while preserving critical data
- **Performance Optimization**: Maintain peak operational speed through intelligent memory allocation
- **Adaptive Intelligence**: Focus on relevant information with context-aware memory prioritization
- **Framework Agnostic**: Works with popular AI frameworks and agent architectures

## 🎯 Why Mesh?

Traditional AI agent frameworks struggle with memory bloat and isolated agent knowledge. Mesh solves these fundamental problems by introducing two industry firsts:

1. **Memory Filtering**: Automatically identifies and removes obsolete memories
2. **Memory Sharing**: Enables agents to share relevant memories across the network

This creates a clean, connected memory layer that allows AI agents to forget what doesn't matter and sync what does.

## 📦 Installation

### Prerequisites

- Python 3.8 or higher
- pip package manager

### Install from PyPI

```bash
pip install mesh-ai
```

### Install from Source

```bash
git clone https://github.com/meshframework/mesh.git
cd mesh
pip install -e .
```

### Development Installation

```bash
git clone https://github.com/meshframework/mesh.git
cd mesh
pip install -e ".[dev]"
```

## 🔧 Quick Start

### Basic Memory Filtering

```python
from mesh import MemoryManager, Agent

# Create a memory manager with filtering enabled
memory_manager = MemoryManager(
    max_memory_size=1000,
    filter_threshold=0.3,
    cleanup_interval=60  # seconds
)

# Create an agent with Mesh memory management
agent = Agent(
    name="assistant",
    memory_manager=memory_manager
)

# Add memories - Mesh will automatically filter obsolete ones
agent.remember("User prefers morning meetings")
agent.remember("Project deadline is next Friday")
agent.remember("Temporary note: coffee machine broken")  # Will be filtered out

# Retrieve relevant memories
relevant_memories = agent.recall("meeting preferences")
print(relevant_memories)
```

### Memory Sharing Between Agents

```python
from mesh import SharedMemoryNetwork, Agent

# Create a shared memory network
network = SharedMemoryNetwork()

# Create agents that share memory
agent1 = Agent("researcher", network=network)
agent2 = Agent("analyst", network=network)

# Agent1 learns something important
agent1.remember("Market trend: AI adoption increasing 40% YoY", importance=0.9)

# Agent2 can access this shared memory
shared_knowledge = agent2.recall("market trends")
print(f"Agent2 knows: {shared_knowledge}")
```

### Advanced Configuration

```python
from mesh import MemoryManager, FilterPolicy, SharingPolicy

# Custom memory filtering policy
filter_policy = FilterPolicy(
    time_decay=0.1,          # How quickly memories fade
    relevance_threshold=0.4,  # Minimum relevance to keep
    max_age_days=30,         # Maximum age before auto-deletion
    preserve_keywords=["important", "critical", "remember"]
)

# Custom sharing policy
sharing_policy = SharingPolicy(
    share_threshold=0.6,     # Minimum importance to share
    network_scope="global",  # Share across all agents
    encryption=True          # Encrypt shared memories
)

# Create advanced memory manager
memory_manager = MemoryManager(
    filter_policy=filter_policy,
    sharing_policy=sharing_policy,
    analytics=True  # Enable memory analytics
)
```

### Integration with Popular Frameworks

#### LangChain Integration

```python
from mesh import MeshLangChainMemory
from langchain.agents import create_react_agent

# Create Mesh-powered LangChain memory
mesh_memory = MeshLangChainMemory(
    filter_enabled=True,
    sharing_enabled=True
)

# Use with LangChain agent
agent = create_react_agent(
    llm=your_llm,
    tools=your_tools,
    memory=mesh_memory
)
```

#### OpenAI Assistant Integration

```python
from mesh import MeshOpenAIMemory
import openai

# Create OpenAI client with Mesh memory
mesh_memory = MeshOpenAIMemory(agent_id="assistant-1")

# Your OpenAI assistant calls
def process_conversation(messages):
    # Mesh automatically filters and manages memory
    relevant_context = mesh_memory.get_relevant_context(messages)
    
    response = openai.ChatCompletion.create(
        model="gpt-4",
        messages=relevant_context + messages
    )
    
    # Store important information
    mesh_memory.store_interaction(messages, response)
    return response
```

## 📊 Memory Analytics

Monitor your agent's memory performance:

```python
from mesh import MemoryAnalytics

analytics = MemoryAnalytics(memory_manager)

# Get memory statistics
stats = analytics.get_stats()
print(f"Memory utilization: {stats.utilization}%")
print(f"Filtered memories: {stats.filtered_count}")
print(f"Shared memories: {stats.shared_count}")

# Visualize memory trends
analytics.plot_memory_trends()
```

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=mesh

# Run specific test category
pytest tests/test_memory_filtering.py
pytest tests/test_memory_sharing.py
```

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Install development dependencies: `pip install -e ".[dev]"`
4. Make your changes
5. Run tests: `pytest`
6. Run linting: `black . && flake8`
7. Commit your changes: `git commit -m 'Add amazing feature'`
8. Push to your branch: `git push origin feature/amazing-feature`
9. Open a Pull Request

### Reporting Issues

- Use GitHub Issues for bug reports and feature requests
- Include detailed reproduction steps for bugs
- Provide system information and Mesh version

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│     Agent A     │    │     Agent B     │    │     Agent C     │
│                 │    │                 │    │                 │
│  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │
│  │   Memory  │  │    │  │   Memory  │  │    │  │   Memory  │  │
│  │  Manager  │  │    │  │  Manager  │  │    │  │  Manager  │  │
│  └─────┬─────┘  │    │  └─────┬─────┘  │    │  └─────┬─────┘  │
└────────┼────────┘    └────────┼────────┘    └────────┼────────┘
         │                      │                      │
         └──────────────────────┼──────────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │  Shared Memory Network │
                    │                        │
                    │  ┌─────────────────┐   │
                    │  │ Memory Filtering │   │
                    │  │ Memory Sharing   │   │
                    │  │ Analytics       │   │
                    │  └─────────────────┘   │
                    └────────────────────────┘
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Made with ❤️ by the Mesh AI Team**

*Forget everything you know about agent memory. Experience the future of AI with Mesh.*