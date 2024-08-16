// package_test demonstrates various Go language features in the context of software testing
package package_test

// Different ways of importing
import (
	"context"
	"fmt"
	"math"
	u "net/url"
	"reflect"
	"sync"
	"testing"
	"time"
	"unsafe"
)

// Constants demonstration
const (
	pi        = 3.14159
	e         = 2.71828
	debugMode = true
)

// Iota demonstration
const (
	low = iota
	medium
	high
)

// Custom error type
type TestError struct {
	message string
}

func (e *TestError) Error() string {
	return e.message
}

// Interface demonstration
type Testable interface {
	Test() bool
}

// Struct demonstration
type TestCase struct {
	Name     string      `json:"name,omitempty" db:"name"`
	Input    interface{} `json:"input" db:"input"`
	Expected interface{} `json:"expected" db:"expected"`
	unequal  bool        // Unexported field
}

// Embedded struct
type ExtendedTestCase struct {
	TestCase
	timeout time.Duration
}

// Function type
type TestFunction func(*testing.T)

// Map type
var testResults = make(map[string]bool)

// Channel type
var testChannel = make(chan TestCase, 10)

// Slice type
var testCases []TestCase

// Array type
var fixedTestCases [5]TestCase

// Pointer type
type TestPointer *TestCase

// Generic type
type GenericPair[T any] struct {
	First  T
	Second T
}

// Generic function
func Max[T int | float64](a, b T) T {
	if a > b {
		return a
	}
	return b
}

// Method declaration
func (tc *TestCase) Run(t *testing.T) {
	// Method implementation
}

// Function declaration with variadic parameter
func runTests(t *testing.T, tests ...TestFunction) {
	for _, test := range tests {
		test(t)
	}
}

// Main test function
func TestMain(m *testing.M) {
	// Setup
	defer func() {
		// Defer statement
		fmt.Println("Cleanup after tests")
	}()

	// Run tests
	m.Run()
}

// Sample test function
func TestSample(t *testing.T) {
	// Short variable declaration
	x := 42

	// If statement
	if x > 0 {
		t.Log("Positive number")
	} else {
		t.Error("Non-positive number")
	}

	// Switch statement
	switch {
	case x < 0:
		t.Error("Negative number")
	case x == 0:
		t.Error("Zero")
	default:
		t.Log("Positive number")
	}

	// For loop
	sum := 0
	for i := 1; i <= 10; i++ {
		sum += i
	}

	// Assert
	if sum != 55 {
		t.Errorf("Expected sum to be 55, got %d", sum)
	}

	// Goroutine and channel usage
	go func() {
		testChannel <- TestCase{Name: "async test", Input: 1, Expected: 1}
	}()

	// Select statement
	select {
	case tc := <-testChannel:
		t.Logf("Received test case: %s", tc.Name)
	case <-time.After(1 * time.Second):
		t.Error("Timeout waiting for test case")
	}

	// Panic and recover
	defer func() {
		if r := recover(); r != nil {
			t.Log("Recovered from panic:", r)
		}
	}()

	// Intentional panic
	if debugMode {
		panic("This is a debug panic")
	}
}

// Benchmark function
func BenchmarkSample(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = math.Sqrt(float64(i))
	}
}

// Example function
func ExampleTestCase_Run() {
	tc := TestCase{Name: "example", Input: 2, Expected: 4}
	fmt.Printf("Running test case: %s\n", tc.Name)
	// Output: Running test case: example
}

// Type assertion and type switch
func processValue(v interface{}) {
	switch x := v.(type) {
	case int:
		fmt.Printf("Integer: %d\n", x)
	case string:
		urlValue, err := u.Parse(x)
		if err == nil {
			fmt.Printf("URL: %s\n", urlValue)
		} else {
			fmt.Printf("String: %s\n", x)
		}
	default:
		fmt.Printf("Unknown type: %T\n", x)
	}
}

// Closure
func createMultiplier(factor int) func(int) int {
	return func(x int) int {
		return x * factor
	}
}

// Complex numbers
var complexNumber = 3 + 4i

// Rune and string literals
var (
	runeValue    = '世'
	rawString    = `This is a "raw" string`
	interpString = "Interpolated \n string"
)

// Bit operations
const (
	read = 1 << iota
	write
	execute
)

// Mutex usage
var (
	mu             sync.Mutex
	sharedResource int
)

// Context usage
func longRunningOperation(ctx context.Context) error {
	select {
	case <-time.After(5 * time.Second):
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

// Reflection usage
func inspectType(x interface{}) {
	t := reflect.TypeOf(x)
	fmt.Printf("Type: %v, Kind: %v\n", t, t.Kind())
}

// Unsafe pointer usage (use with caution)
func unsafePointerExample() {
	x := [4]int{1, 2, 3, 4}
	p := unsafe.Pointer(&x)
	fmt.Printf("First element: %d\n", *(*int)(p))
}

// Init function
func init() {
	fmt.Println("Initializing package")
}

// Main function (uncomment if using as a standalone program)
// func main() {
// 	fmt.Println("Running tests...")
// 	testing.Main(func(pat, str string) (bool, error) { return true, nil },
// 		[]testing.InternalTest{{Name: "TestSample", F: TestSample}},
// 		[]testing.InternalBenchmark{{Name: "BenchmarkSample", F: BenchmarkSample}},
// 		[]testing.InternalExample{{Name: "ExampleTestCase_Run", F: ExampleTestCase_Run}})
// }

// Fallthrough demonstration
func fallThroughExample(x int) string {
	switch x {
	case 0:
		fallthrough
	case 1:
		return "Low"
	case 2:
		return "Medium"
	default:
		return "High"
	}
}

// Label and goto demonstration
func labelAndGotoExample() {
	i := 0
Loop:
	if i < 5 {
		i++
		goto Loop
	}
}

// ... (end of file)

/*
	A multi-line comment
	spanning multiple lines.
*/

type (
	Point struct{ x, y float64 }
	polar Point
)

type TreeNode struct {
	left, right *TreeNode
	value       any
}

type Block interface {
	BlockSize() int
	Encrypt(src, dst []byte)
	Decrypt(src, dst []byte)
}

func add(a, b int) int {
	return a + b
}

type Rectangle struct {
	width, height float64
}

func (r Rectangle) Area() float64 {
	mul := func(a, b float64) float64 {
		return a * b
	}

	return mul(r.width, r.height)
}

func variadic(nums ...int) int {
	total := 0
	for _, num := range nums {
		total += num
	}
	return total
}

func higherOrder(f func(int) int, x int) int {
	return f(x)
}

func closure() func() int {
	count := 0
	return func() int {
		count++
		return count
	}
}

type (
	rectangles = []*Rectangle
	Polar      = polar
)
