# This is a single-line comment
# It starts with a hash and continues to the end of the line.

/*
This is a multi-line comment block.
It starts with a slash and a star, and ends with a star and a slash.
Multi-line comments are useful for longer explanations or
temporarily disabling a block of code.
*/

# Example resource block with different comment styles
resource "aws_instance" "example" {
  ami           = "ami-0c55b159cbfafe1f0" # Inline single-line comment
  instance_type = "t2.micro"              // Another style of single-line comment

  /*
    Multi-line comment above a key-value pair
  */
  availability_zone = "us-west-2a"

  tags = {
    Name = "ExampleInstance"
    # Comment describing a specific tag entry
    Environment = "Dev"
  }
}

# Another top-level single-line comment
resource "aws_security_group" "example" {
  name        = "example_security_group"
  description = "A security group for our example instance."

  // Inline comment before a block of code
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    /* Multi-line comment within a block */
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
    # Single-line comment at the end of a block
  }
}

# Example output with a comment above
output "instance_ip_addr" {
  /* This output shows the public IP address of the instance */
  value = aws_instance.example.public_ip
}

/*
End of Terraform configuration example file.
Use comments effectively to make your code more understandable and maintainable!
*/
