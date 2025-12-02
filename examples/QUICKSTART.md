# NeuroScript Training Quickstart

## 5-Minute Tutorial: Train Your First Neural Network

### Step 1: Activate Environment

```bash
source ~/.venv_ai/bin/activate
```

### Step 2: Run the XOR Example

```bash
python examples/test_xor_training.py
```

That's it! You just:
- ✅ Compiled NeuroScript to PyTorch
- ✅ Trained a neural network
- ✅ Tested inference
- ✅ Saved checkpoints

### What Just Happened?

```
examples/01-xor.ns          # Your NeuroScript architecture
      ↓
  [Rust Compiler]
      ↓
   XOR.py                   # Generated PyTorch module
      ↓
  [Python Runner]
      ↓
Trained XOR Model           # Ready for inference!
```

### Step 3: Try Your Own Model

**1. Write NeuroScript** (`my_model.ns`)

```neuroscript
use core,nn/*

neuron MyClassifier():
    in: [batch, 10]
    out: [batch, 2]
    graph:
        in ->
            Linear(10, 20)
            ReLU()
            Linear(20, 2)
            out
```

**2. Create Training Data** (`my_data.jsonl`)

```jsonl
{"input": [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0], "target": [1.0, 0.0]}
{"input": [0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0], "target": [0.0, 1.0]}
```

**3. Create Config** (`my_config.yml`)

```yaml
model:
  neuron: MyClassifier
  file: my_model.ns

data:
  train: my_data.jsonl

training:
  epochs: 100
  batch_size: 2
```

**4. Train**

```python
from neuroscript_runtime.runner_v2 import train_from_config
from pathlib import Path

# Compile first:
# ./target/release/neuroscript --codegen MyClassifier my_model.ns --output my_model.py

from my_model import MyClassifier

model = MyClassifier()
runner = train_from_config(model, Path("my_config.yml"))

# Inference
import torch
result = runner.infer(torch.randn(1, 10))
print(result)
```

## Examples Available

| Example | Description | Difficulty |
|---------|-------------|------------|
| `01-xor.ns` | XOR function - classic non-linear problem | ⭐ Beginner |
| `02-palindrome.ns` | Palindrome detection with attention | ⭐⭐ Intermediate |
| `03-addition.ns` | Learn arithmetic operations | ⭐ Beginner |
| `transformer_from_stdlib.ns` | GPT-2 style transformer | ⭐⭐⭐ Advanced |

## Common Tasks

### Compile NeuroScript to PyTorch

```bash
./target/release/neuroscript --codegen NeuronName file.ns --output model.py
```

### Train from Config

```python
from neuroscript_runtime.runner_v2 import train_from_config
from my_model import MyModel

runner = train_from_config(MyModel(), "config.yml")
```

### Manual Training Loop

```python
from neuroscript_runtime.runner_v2 import NeuroScriptRunner, TrainingConfig
from neuroscript_runtime.defaults import JSONLDataLoader

config = TrainingConfig(epochs=100, batch_size=32, lr=0.01)
train_loader = JSONLDataLoader("train.jsonl", batch_size=32)

runner = NeuroScriptRunner(model, config, train_loader)
runner.train()
```

### Load Checkpoint & Infer

```python
runner.load_checkpoint(Path("checkpoints/best.pt"))
result = runner.infer(torch.randn(1, input_size))
```

## Configuration Reference

### Minimal Config

```yaml
model:
  neuron: ModelName
  file: model.ns

data:
  train: train.jsonl

training:
  epochs: 10
```

### Full Config

```yaml
model:
  neuron: ModelName
  file: model.ns
  params:
    dim: 512

data:
  train: train.jsonl
  val: val.jsonl

training:
  batch_size: 32
  epochs: 10
  optimizer: adam
  lr: 0.001
  loss: mse
  max_grad_norm: 1.0
  log_every: 100
  eval_every: 500
  checkpoint_dir: ./checkpoints
  save_every: 1000
```

## Extending the System

### Add Custom Data Loader

```python
from neuroscript_runtime.contracts import DataLoaderContract, ContractRegistry

class MyLoader(DataLoaderContract):
    # Implement required methods
    pass

ContractRegistry.register_dataloader("myformat", MyLoader)
```

### Add Custom Loss

```python
from neuroscript_runtime.contracts import LossContract, ContractRegistry

class MyLoss(LossContract):
    def compute(self, preds, targets):
        return my_loss_fn(preds, targets)

ContractRegistry.register_loss("myloss", MyLoss)
```

## Troubleshooting

### Error: "Module not found"

```bash
# Make sure you activated the environment
source ~/.venv_ai/bin/activate

# Install the runtime package
pip install -e .
```

### Error: "Neuron not found"

Check your neuron name matches exactly:

```bash
# List available neurons
./target/release/neuroscript examples/01-xor.ns

# Should show: XOR, XORExplicit, XORDeep
```

### Error: "Cannot load data"

Check your JSONL format:

```jsonl
{"input": [1.0, 2.0], "target": [3.0]}
```

Keys must be `"input"` and `"target"`, values must be arrays of numbers.

## Next Steps

1. **Read TRAINING.md** - Complete training guide
2. **Try examples** - `examples/test_xor_training.py`
3. **Build your own** - Create custom architectures
4. **Extend the system** - Add custom data loaders, losses, etc.
5. **Join the community** - Share your implementations!

## Resources

- `TRAINING.md` - Comprehensive training guide
- `CLAUDE.md` - Compiler architecture and development
- `INFERENCE_AND_TRAINING_SUMMARY.md` - System overview
- `neuroscript_runtime/contracts.py` - Extension points
- `neuroscript_runtime/defaults.py` - Default implementations

Happy training! 🚀
