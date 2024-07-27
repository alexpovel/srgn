using System;
using System.Collections.Generic;
using static System.Math;
using Console = System.Console;


[assembly: System.Runtime.CompilerServices.InternalsVisibleTo("TestAssembly")]

namespace SoftwareTesting
{
    public unsafe struct TestResult<T> where T : struct
    {
        public T Value;
        public fixed byte ErrorCode[16];
    }

    [Flags]
    public enum TestStatus : byte
    {
        None = 0,
        Running = 1 << 0,
        Passed = 1 << 1,
        Failed = 1 << 2
    }

    public interface ITestable<out TResult>
    {
        TResult RunTest();
        event EventHandler<TResult> TestCompleted;
    }

    [Serializable]
    public abstract class TestBase : ITestable<TestResult<double>>, IDisposable
    {
        protected internal const double Epsilon = 1e-6;
        public static readonly DateTime TestStartTime = DateTime.Now;

        private TestStatus _status;
        public ref readonly TestStatus Status => ref _status;

        protected TestBase() => _status = TestStatus.None;

        public abstract TestResult<double> RunTest();

        public event EventHandler<TestResult<double>> TestCompleted;

        protected virtual void OnTestCompleted(TestResult<double> result) =>
            TestCompleted?.Invoke(this, result);

        ~TestBase() => Console.WriteLine("Test finalized.");

        public void Dispose()
        {
            GC.SuppressFinalize(this);
        }
    }

    public sealed class PerformanceTest : TestBase
    {
        private readonly Func<double, double> _functionUnderTest;
        private readonly int _iterations;

        public PerformanceTest(Func<double, double> func, int iterations)
        {
            _functionUnderTest = func ?? throw new ArgumentNullException(nameof(func));
            _iterations = iterations > 0 ? iterations : throw new ArgumentOutOfRangeException(nameof(iterations));
        }

        public override TestResult<double> RunTest()
        {
            _status = TestStatus.Running;
            var stopwatch = System.Diagnostics.Stopwatch.StartNew();

            for (int i = 0; i < _iterations; i++)
            {
                _functionUnderTest(i);
            }

            stopwatch.Stop();

            var result = new TestResult<double>
            {
                Value = stopwatch.Elapsed.TotalMilliseconds / _iterations
            };

            _status = TestStatus.Passed;
            OnTestCompleted(result);
            return result;
        }

        public static bool operator ==(PerformanceTest left, PerformanceTest right) =>
            left?._iterations == right?._iterations;

        public static bool operator !=(PerformanceTest left, PerformanceTest right) =>
            !(left == right);

        public override bool Equals(object obj) =>
            obj is PerformanceTest test && this == test;

        public override int GetHashCode() => _iterations;
    }

    public static class TestExtensions
    {
        public static void PrintResult(this TestResult<double> result) =>
            Console.WriteLine($"Test result: {result.Value:F3} ms");
    }

    public delegate void TestLogger(string message);

    public partial class TestRunner
    {
        private readonly List<TestBase> _tests = new List<TestBase>();
        private TestLogger _logger;

        public event EventHandler AllTestsCompleted;

        public TestRunner(TestLogger logger = null) => _logger = logger ?? Console.WriteLine;

        public void AddTest(TestBase test) => _tests.Add(test);

        public async System.Threading.Tasks.Task RunAllTestsAsync()
        {
            foreach (var test in _tests)
            {
                await System.Threading.Tasks.Task.Run(() =>
                {
                    var result = test.RunTest();
                    _logger($"Test completed: {result.Value}");
                });
            }

            AllTestsCompleted?.Invoke(this, EventArgs.Empty);
        }

        partial void OnTestRunnerInitialized();
    }

    public class TestException : Exception
    {
        public TestException(string message) : base(message) { }
    }

    [AttributeUsage(AttributeTargets.Method, AllowMultiple = false)]
    public class BenchmarkAttribute : Attribute
    {
        public int Iterations { get; }
        public BenchmarkAttribute(int iterations) => Iterations = iterations;
    }


    /// <summary>
    /// Some class!
    /// </summary>
    public class Program
    {
        // Some comment.
        public static void /* An inline comment */ Main(string[] args)
        {
            /*
                A block comment.
                It has multiple lines.
            */
            TestRunner runner = new TestRunner();
            runner.AddTest(new PerformanceTest(x => Pow(x, 2), 1000));

            runner.AllTestsCompleted += (sender, e) => Console.WriteLine("All tests completed!");

            _ = runner.RunAllTestsAsync();

            string interpolatedVerbatimString = $@"User {name} has the ID: {user_Id}";
            string interpolatedStringText = $"Found user with ID: {user_Id}";
            string rawStringLiteral = """This is a
raw string
literal""";
            string stringLiteral = "Alice";
            string verbatimStringLiteral = @"C:\Users\Alice\Documents";

            int? nullableInt = 5;
            int nonNullableInt = nullableInt ?? throw new TestException("Unexpected null value");

            var tuple = (Name: "Test", Value: 42);
            Console.WriteLine($"Tuple: {tuple.Name}, {tuple.Value}");

            string interpolatedString = $"The value is {nonNullableInt}";

            object obj = new PerformanceTest(Math.Sin, 100);
            if (obj is PerformanceTest pt && pt._iterations > 50)
            {
                Console.WriteLine("High iteration performance test");
            }

            var anonymousObject = new { Name = "AnonymousTest", Value = 3.14 };

            var query = from test in runner._tests
                        where test is PerformanceTest
                        select test;

            int[] numbers = { 1, 2, 3, 4, 5 };
            var evenNumbers = numbers.Where(n => n % 2 == 0);

            Test:
            Console.WriteLine("Labeled statement");
            goto Test;
        }

        [Benchmark(100)]
        public static unsafe void UnsafeMethod()
        {
            int value = 42;
            int* ptr = &value;
            Console.WriteLine(*ptr);
        }

        public static dynamic TestDynamic(dynamic input) => input.ToString();

        public static void Deconstruct(this (int, string) tuple, out int number, out string text)
        {
            number = tuple.Item1;
            text = tuple.Item2;
        }
    }
}
