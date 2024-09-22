#include <iostream>
#include <vector>
#include <string>
#include <memory>
#include <optional>
#include <variant>
#include <functional>
#include <algorithm>
#include <initializer_list>
#include <tuple>
#include <map>

#include "local_include.hpp"
#include "../remote_include.hpp"

/* Simple struct */
struct Point {
    int x, y;
};

// Class with some members
class Example {
public:
    Example() : value(0) {}
    explicit Example(int val) : value(val) {}

    ~Example() {}

    Example operator+(const Example& other) const {
        return Example(value + other.value);
    }

    static int staticMethod() {
        return 42;
    }

    template<typename T>
    T add(T a, T b) {
        return a + b;
    }

private:
    int value;

protected:
    const char *ident;
};

enum class Color { Red, Green, Blue };

union Converter { const int value; const char *ptr; };

namespace MyNamespace {
    void function() {
        std::cout << "Function in namespace\n";
    }
}
using namespace MyNamespace;

template<typename T>
struct Wrapper {
    T value;
};

template<typename... Args>
void variadicFunction(Args... args) {
    (std::cout << ... << args) << '\n';
}

auto lambda = [](int a, int b) { return a + b; };

std::optional<int> findValue(const std::vector<int>& vec, int value) {
    auto it = std::find(vec.begin(), vec.end(), value);
    if (it != vec.end()) {
        return *it;
    }
    return std::nullopt;
}

std::variant<int, std::string> getVariant(bool flag) {
    if (flag) {
        return 42;
    }
    return "Hello";
}

std::tuple<int, double, std::string> createTuple() {
    return {1, 2.5, "example"};
}

int main() {
    int a = 5;
    if (a > 0) {
        std::cout << "Positive\n";
    } else if (a < 0) {
        std::cout << "Negative\n";
    } else {
        std::cout << "Zero\n";
    }

    for (int i = 0; i < 5; ++i) {
        std::cout << i << " ";
    }
    std::cout << "\n";

    int i = 0;
    while (i < 5) {
        std::cout << i++ << " ";
    }
    std::cout << "\n";

    do {
        std::cout << i-- << " ";
    } while (i > 0);
    std::cout << "\n";

    Example ex1;
    Example ex2(10);
    Example ex3 = ex1 + ex2;

    Point p{3, 4};

    Color color = Color::Red;

    MyNamespace::function();

    Wrapper<int> wrappedValue{10};

    variadicFunction(1, 2.5, "test");

    int sum = lambda(3, 4);

    auto value = findValue({1, 2, 3}, 2);
    if (value) {
        std::cout << "Found: " << *value << "\n";
    }

    auto variantValue = getVariant(true);
    if (std::holds_alternative<int>(variantValue)) {
        std::cout << "Variant holds int: " << std::get<int>(variantValue) << "\n";
    }

    auto [x, y, z] = createTuple();

    std::pair<int, std::string> pairValue{1, "one"};

    std::unique_ptr<Example> ptr = std::make_unique<Example>(20);

    std::vector<int> numbers = {1, 2, 3, 4, 5};
    std::sort(numbers.begin(), numbers.end(), std::greater<>());

    std::map<std::string, int> myMap;
    myMap["one"] = 1;

    try {
        Example ex4(20);
    } catch (const std::exception& e) {
        std::cout << "Exception: " << e.what() << "\n";
    }

    switch (color) {
        case Color::Red:
            std::cout << "Red\n";
        case Color::Blue:
            std::cout << "Blue\n";
        case Color::Green:
            std::cout << "Green\n";
        default:
            std::cout << "Default\n";
    }

    return 0;
}
