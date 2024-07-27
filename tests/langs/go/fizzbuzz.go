package package_test

import "fmt"

// fizzBuzz prints the numbers from 1 to a specified limit.
// For multiples of 3, it prints "Fizz" instead of the number,
// for multiples of 5, it prints "Buzz", and for multiples of both 3 and 5,
// it prints "FizzBuzz".
func fizzBuzz(limit int) {
	for i := 1; i <= limit; i++ {
		switch {
		case i%3 == 0 && i%5 == 0:
			fmt.Println("FizzBuzz")
		case i%3 == 0:
			fmt.Println("Fizz")
		case i%5 == 0:
			fmt.Println("Buzz")
		default:
			fmt.Println(i)
		}
	}
}

func main() {
	// Run the FizzBuzz function for numbers from 1 to 100
	fizzBuzz(100)
}
