using System.Linq;

public class UserService
{
    private readonly AppDbContext _dbContext;

    
    
    
    
    public UserService(AppDbContext dbContext)
    {
        _dbContext  = dbContext;
    }

    
    
    
    
    public void DoWork()
    {
        _dbContext.Database.EnsureCreated(); 

        _dbContext.Users.Add(new User  { Name = "Alice" });

        
        _dbContext.SaveChanges();

        var user = _dbContext.Users.Where( u => u.Name == "Alice").FirstOrDefault();

        
        if (user  != null)
        {
            System.Console.WriteLine($"Found user with ID: {user.Id}");
        }
    }
}
