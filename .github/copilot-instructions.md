# Copilot Code Generation Instructions

This document outlines the requirements and principles that should be followed when generating code for this project.

## Core Principles

### 1. Single Responsibility Principle (SRP)
- Each class/module/component should have only one reason to change
- Functions should do one thing and do it well
- Avoid mixing different levels of abstraction in the same component
- Split large classes/functions into smaller, focused ones when they handle multiple concerns

### 2. Open/Closed Principle (OCP)
- Software entities should be open for extension but closed for modification
- Use interfaces and abstract classes to define contracts
- Implement new functionality through inheritance or composition rather than modifying existing code
- Use strategy pattern or plugins when behavior needs to be extended

### 3. Liskov Substitution Principle (LSP)
- Derived classes must be substitutable for their base classes
- Ensure inherited classes follow the contract of the base class
- Don't violate method pre-conditions and post-conditions in derived classes
- Maintain expected behavior when using polymorphism

### 4. Interface Segregation Principle (ISP)
- Clients should not be forced to depend on interfaces they don't use
- Create specific, focused interfaces rather than large, general-purpose ones
- Split large interfaces into smaller, more specific ones
- Keep interfaces minimal and cohesive

### 5. Dependency Inversion Principle (DIP)
- High-level modules should not depend on low-level modules
- Both should depend on abstractions
- Use dependency injection where appropriate
- Define clear interface contracts between layers

### 6. Separation of Concerns (SoC)
- Keep different aspects of functionality separated
- Maintain clear boundaries between:
  - Business logic
  - Data access
  - Presentation logic
  - Configuration
- Use layered architecture patterns appropriately

### 7. Testing Requirements
- Every new class must have corresponding unit tests
- Every new method must have relevant test cases
- Every new component must have component tests
- Test coverage should include:
  - Happy path scenarios
  - Edge cases
  - Error conditions
- Tests should be:
  - Independent
  - Repeatable
  - Clear and readable
  - Fast-running

### 8. Code Simplicity and Maintainability
- Write clean, self-documenting code
- Follow the DRY (Don't Repeat Yourself) principle
- Keep methods short and focused
- Use meaningful names for variables, methods, and classes
- Avoid unnecessary complexity
- Include appropriate comments for complex logic
- Maintain consistent code formatting
- Keep cyclomatic complexity low
- Limit nesting levels

## Implementation Guidelines

1. **Documentation**
   - Add JSDoc/docstring comments for public APIs
   - Include examples in documentation where appropriate
   - Document any assumptions or prerequisites

2. **Error Handling**        
   - Use appropriate error handling mechanisms
   - Create custom error types when needed
   - Provide meaningful error messages

3. **Code Organization**
   - Group related functionality together
   - Use appropriate design patterns
   - Maintain a clear project structure

4. **Performance Considerations**
   - Consider time and space complexity
   - Optimize only when necessary
   - Document performance-critical sections

5. **Security**
   - Follow security best practices
   - Validate inputs
   - Handle sensitive data appropriately

6. **Platform-Specific Requirements**
   - On Windows systems, use `;;` instead of `&&` for command chaining
   - Example: `cd frontend ;; npm test` instead of `cd frontend && npm test`
   - This applies to all shell commands, batch files, and npm scripts

Remember: These principles should be applied pragmatically. While we strive to follow them, there may be situations where trade-offs are necessary. Document any deviations from these principles and provide justification.