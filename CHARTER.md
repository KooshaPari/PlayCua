# bare-cua Charter

## 1. Mission Statement

**bare-cua** is a minimal, foundational CUDA utility and abstraction layer designed to provide streamlined GPU computing primitives for the Phenotype ecosystem. The mission is to offer a thin, efficient interface to NVIDIA GPU capabilities—enabling high-performance computing, parallel processing, and machine learning acceleration without the overhead of heavy frameworks.

The project exists to be the lightweight GPU foundation—providing essential CUDA operations, memory management, and kernel utilities that other Phenotype components can build upon for accelerated computation.

---

## 2. Tenets (Unless You Know Better Ones)

### Tenet 1: Minimal Overhead

Thin abstraction. Direct CUDA when needed. No heavy framework. Pay for what you use. Efficiency first.

### Tenet 2. Explicit Resource Management

GPU memory explicit. Allocation visible. Deallocation clear. RAII patterns. No hidden allocations.

### Tenet 3. Safety Through Types

CUDA errors caught. Memory safety. Type-safe kernels. Compile-time checks where possible.

### Tenet 4. Composable Primitives

Small utilities compose. Kernel helpers. Memory utilities. Stream management. Build complex from simple.

### Tenet 5. Multi-Language Support

Rust interface. Python bindings. C++ foundation. Language appropriate APIs. FFI efficient.

### Tenet 6. Observable Performance

Kernel timing. Memory usage. Stream utilization. Profiling hooks. Understand GPU usage.

### Tenet 7. Graceful CPU Fallback

GPU unavailable? CPU fallback. Automatic or explicit. No hard dependency on GPU. Resilient.

---

## 3. Scope & Boundaries

### In Scope

**Core CUDA:**
- Context management
- Stream management
- Memory allocation (device, unified, pinned)
- Memory copy utilities
- Error handling

**Kernel Utilities:**
- Launch configuration
- Grid/block helpers
- Template kernels
- Reduction utilities
- Scan primitives

**Memory Management:**
- Pool allocators
- Memory pools
- Fragmentation management
- Async allocation

**Multi-GPU:**
- Device enumeration
- Peer access
- Multi-device coordination
- Load balancing helpers

**Bindings:**
- Rust interface
- Python bindings (PyO3)
- C interface

### Out of Scope

- ML framework (use PyTorch, JAX)
- Linear algebra (use cuBLAS, cuSOLVER)
- Deep learning primitives (use cuDNN)
- Graphics (use Vulkan, OpenGL)

### Boundaries

- Foundation layer, not framework
- Primitives, not algorithms
- GPU utility, not GPU application
- Building blocks, not solutions

---

## 4. Target Users & Personas

### Primary Persona: GPU Developer Greg

**Role:** Developer writing GPU code
**Goals:** Efficient GPU utilization, clean abstractions
**Pain Points:** CUDA verbosity, memory management
**Needs:** Clean API, memory management, utilities
**Tech Comfort:** Very high, CUDA expert

### Secondary Persona: ML Engineer Mel

**Role:** ML engineer optimizing performance
**Goals:** Fast training, efficient inference
**Pain Points:** Framework overhead, custom kernels
**Needs:** Custom kernel support, memory efficiency
**Tech Comfort:** High, ML focus

### Tertiary Persona: Rust Developer Ray

**Role:** Rust developer using GPU
**Goals:** Rust + GPU integration
**Pain Points:** Rust CUDA ecosystem immature
**Needs:** Safe Rust interface, good bindings
**Tech Comfort:** Very high, Rust expert

---

## 5. Success Criteria (Measurable)

### Performance

- **Overhead:** <5% overhead vs. raw CUDA
- **Memory Efficiency:** Efficient allocation patterns
- **Kernel Launch:** Minimal launch overhead
- **Copy Speed:** Optimal memory transfers

### Quality

- **Safety:** Zero memory safety issues
- **Error Handling:** 100% of CUDA errors handled
- **Test Coverage:** 90%+ code coverage
- **Documentation:** 100% of public API documented

### Adoption

- **Integration:** Used by 3+ Phenotype projects
- **Binding Quality:** 4.0/5+ binding satisfaction
- **Build Success:** 95%+ successful builds

---

## 6. Governance Model

### Component Organization

```
bare-cua/
├── core/            # Core CUDA utilities
├── memory/          # Memory management
├── kernel/          # Kernel utilities
├── multi_gpu/       # Multi-GPU support
├── bindings/        # Language bindings
└── tests/           # Test suite
```

### Development Process

**New Primitives:**
- Performance testing
- Safety review
- Multi-language testing

**Breaking Changes:**
- Performance regression testing
- Migration guide
- Version bump

---

## 7. Charter Compliance Checklist

### For New Primitives

- [ ] Performance tested
- [ ] Safety verified
- [ ] Multi-language tested
- [ ] Documentation complete

### For Breaking Changes

- [ ] Performance regression tested
- [ ] Migration guide
- [ ] Version bumped

---

## 8. Decision Authority Levels

### Level 1: Maintainer Authority

**Scope:** Bug fixes, optimizations
**Process:** Maintainer approval

### Level 2: Core Team Authority

**Scope:** New primitives, bindings
**Process:** Team review

### Level 3: Technical Steering Authority

**Scope:** Breaking changes, architecture
**Process:** Steering approval

### Level 4: Executive Authority

**Scope:** Strategic direction
**Process:** Executive approval

---

*This charter governs bare-cua, the foundational CUDA utilities. GPU power through clean abstractions.*

*Last Updated: April 2026*
*Next Review: July 2026*
